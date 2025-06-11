use std::{
    alloc::{alloc, Layout},
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::BTreeMap,
    future::Future,
    pin::Pin,
    ptr::{addr_of_mut, null_mut},
    sync::Arc,
    time::Duration,
    vec,
};

use libc::pollfd;
use tokio::{
    sync::{
        mpsc::{UnboundedReceiver, UnboundedSender},
        oneshot::{error::TryRecvError, Sender},
        RwLock,
    },
    task::JoinSet,
};
use tokio_util::sync::CancellationToken;

use crate::{
    error::Error,
    libusb::{
        libusb_close, libusb_context, libusb_device, libusb_error_LIBUSB_SUCCESS, libusb_exit,
        libusb_get_next_timeout, libusb_get_pollfds, libusb_handle_events_timeout,
        libusb_hotplug_callback_handle, libusb_hotplug_deregister_callback, libusb_hotplug_event,
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED,
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT,
        libusb_hotplug_flag_LIBUSB_HOTPLUG_ENUMERATE, libusb_hotplug_register_callback,
        libusb_init_context, libusb_set_pollfd_notifiers, timeval, LIBUSB_HOTPLUG_MATCH_ANY,
    },
    usb::device::UsbDevice,
};

use super::{
    context::{UsbContext, UsbContextRef},
    device::UsbDeviceRef,
    event::{
        EventHandler, UsbEvent, UsbFileDescriptorEventData, UsbHotplugEventData,
        UsbHotplugEventMask, UsbHotplugEventType,
    },
    guard::Guard,
};
pub struct UsbListener;
impl UsbListener {
    pub fn builder<'a>() -> UsbListenerBuilder<'a> {
        UsbListenerBuilder::default()
    }
}

#[derive(Clone, Default)]
pub struct UsbListenerBuilder<'a> {
    handlers: Vec<(UsbHotplugEventMask, Arc<EventHandler<'a>>)>,
    poll_rate: Duration,
}

impl UsbListenerBuilder<'static> {
    pub fn on_event(
        &mut self,
        event_filter: impl Into<UsbHotplugEventMask>,
        handler: EventHandler<'static>,
    ) {
        self.handlers
            .push((event_filter.into(), Arc::new(handler.into())));
    }
    pub fn build(
        self,
    ) -> Result<
        (
            impl Future<Output = ()> + 'static,
            tokio::sync::oneshot::Sender<()>,
        ),
        Error,
    > {
        fn create_event_handler(
            fds: Arc<RwLock<BTreeMap<i32, i16>>>,
            receiver: UnboundedReceiver<UsbEvent>,
            error_sender: Arc<tokio::sync::mpsc::Sender<Error>>,
            handlers: Cow<'static, [(UsbHotplugEventMask, Arc<EventHandler<'static>>)]>,
        ) -> impl Future<Output = ()> + 'static {
            async move {
                let mut receiver = receiver;
                while let Some(event) = receiver.recv().await {
                    match event {
                        UsbEvent::FileDescriptor(event) => {
                            let mut fds = fds.write().await;
                            match event {
                                UsbFileDescriptorEventData::Add(fd, events) => {
                                    fds.insert(fd, events);
                                }
                                UsbFileDescriptorEventData::Remove(fd) => {
                                    fds.remove(&fd);
                                }
                            }
                        }
                        UsbEvent::Hotplug(UsbHotplugEventData(context, device, event_type)) => {
                            let device = Arc::new(UsbContext(context.0, UsbDevice(device.0)));
                            for (filter, handler) in handlers.into_iter() {
                                if filter.matches(event_type) {
                                    let handler = handler.clone();
                                    let error_sender = error_sender.clone();
                                    let device = device.clone();
                                    tokio::spawn(async move {
                                        let error_sender = error_sender;
                                        if let Err(err) = handler(&device, event_type) {
                                            error_sender.send(err);
                                        }
                                    });
                                }
                            }
                        }
                        UsbEvent::Close => {
                            receiver.close();
                        }
                    }
                }
            }
        }
        fn create_poll_handler(
            context: Arc<UsbContext<()>>,
            fds: Arc<RwLock<BTreeMap<i32, i16>>>,
            event_channel: Arc<UnboundedSender<UsbEvent>>,
            poll_rate: Duration,
        ) -> (impl Future<Output = ()> + 'static, Sender<()>) {
            let (send, mut receive) = tokio::sync::oneshot::channel::<()>();

            let future = async move {
                let mut tv = timeval {
                    tv_sec: poll_rate.as_secs() as i64,
                    tv_usec: 0,
                };
                let mut zero_tv = timeval::default();

                while receive
                    .try_recv()
                    .is_err_and(|value| value == TryRecvError::Empty)
                {
                    unsafe {
                        let context = context.clone();
                        let timeout = { libusb_get_next_timeout(**context, &mut tv) };
                        let mut fds_entries = {
                            let fds = fds.read().await;
                            fds.iter()
                                .map(|(fd, events)| pollfd {
                                    fd: *fd,
                                    events: *events,
                                    revents: 0i16,
                                })
                                .collect::<Vec<_>>()
                        };

                        let fds_len = fds_entries.len();
                        libc::poll(fds_entries.as_mut_ptr(), fds_len as u64, timeout);
                        if fds_entries.iter().any(|fd| fd.revents != 0) {
                            if libusb_handle_events_timeout(**context, &mut zero_tv) != 0 {
                                return;
                            }
                        }
                    }
                }
                event_channel.send(UsbEvent::Close);
            };
            (future, send)
        }
        let handlers = self.handlers.into();
        unsafe {
            let mut hotplug_callback_handle: libusb_hotplug_callback_handle = 0;
            let (event_send, event_receive) = tokio::sync::mpsc::unbounded_channel();
            let event_send = Arc::new(event_send);
            let (error_send, error_receive) = tokio::sync::mpsc::channel(128);
            let error_send = Arc::new(error_send);

            let mut context = alloc(Layout::new::<libusb_context>()) as *mut libusb_context;
            libusb_init_context(addr_of_mut!(context), null_mut(), 0);
            let result = libusb_hotplug_register_callback(
                context,
                libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED as i32
                    | libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT as i32,
                libusb_hotplug_flag_LIBUSB_HOTPLUG_ENUMERATE as i32,
                LIBUSB_HOTPLUG_MATCH_ANY,
                LIBUSB_HOTPLUG_MATCH_ANY,
                LIBUSB_HOTPLUG_MATCH_ANY,
                Some(libusb_hotplug),
                Arc::into_raw(event_send.clone()) as *mut std::ffi::c_void,
                &mut hotplug_callback_handle,
            );
            if result != libusb_error_LIBUSB_SUCCESS {
                return Err(Error::RegisterCallback(result));
            }
            libusb_set_pollfd_notifiers(
                context,
                Some(libusb_fds_added),
                Some(libusb_fds_removed),
                Arc::into_raw(event_send.clone()) as *mut std::ffi::c_void,
            );
            let mut initial = BTreeMap::<i32, i16>::new();

            unsafe {
                let current_fds = libusb_get_pollfds(context);
                let mut index = 0;
                loop {
                    let fd = *current_fds.offset(index);
                    if fd.is_null() {
                        break;
                    } else {
                        let fd = *fd;
                        initial.insert(fd.fd, fd.events);
                        index += 1;
                    }
                }
            }
            let fds = Arc::new(RwLock::new(initial));
            let context = Arc::new(UsbContext(context, ()));
            let event_future =
                create_event_handler(fds.clone(), event_receive, error_send.clone(), handlers);
            let (poll_future, close_signal) =
                create_poll_handler(context.clone(), fds, event_send.clone(), self.poll_rate);
            Ok((
                async move {
                    let context = context.clone();
                    let mut futures = JoinSet::new();
                    futures.spawn(event_future);
                    futures.spawn(poll_future);
                    futures.join_all().await;
                    libusb_hotplug_deregister_callback(**context, hotplug_callback_handle);
                    libusb_exit(**context);
                },
                close_signal,
            ))
        }
    }
    pub async fn serve(self) -> Result<(), Error> {
        let (future, close) = self.build()?;
        let ctrlc_cancel = CancellationToken::new();
        let ctrlc_cancel_1 = ctrlc_cancel.clone();
        let ctrlc_handler = tokio::spawn(async move {
            tokio::select! {
                io = tokio::signal::ctrl_c() => {
                    match io {
                        Ok(_) => { let _ = close.send(()); },
                        err => { return err }
                    }
                }
                _ = ctrlc_cancel_1.cancelled() => { }
            }
            Ok::<_, std::io::Error>(())
        });
        let _ = tokio::spawn(async move {
            future.await;
            ctrlc_cancel.cancel();
            ctrlc_handler.await;
        })
        .await;
        Ok(())
    }
}
extern "C" fn libusb_hotplug(
    context: *mut libusb_context,
    device: *mut libusb_device,
    event: libusb_hotplug_event,
    user_data: *mut ::std::os::raw::c_void,
) -> std::os::raw::c_int {
    let user_data = unsafe {
        Arc::increment_strong_count(user_data);
        Arc::from_raw(user_data as *mut tokio::sync::mpsc::UnboundedSender<UsbEvent>)
    };
    let event = match event {
        #[allow(non_upper_case_globals)]
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_LEFT => UsbHotplugEventType::DeviceLeft,
        #[allow(non_upper_case_globals)]
        libusb_hotplug_event_LIBUSB_HOTPLUG_EVENT_DEVICE_ARRIVED => {
            UsbHotplugEventType::DeviceArrived
        }
        _ => {
            return 0;
        }
    };
    return match user_data.send(UsbEvent::Hotplug(UsbHotplugEventData(
        UsbContextRef(context),
        UsbDeviceRef(device),
        event,
    ))) {
        Ok(_) => 0,
        Err(_) => 1,
    };
}
extern "C" fn libusb_fds_added(
    fd: std::os::raw::c_int,
    events: std::os::raw::c_short,
    user_data: *mut ::std::os::raw::c_void,
) {
    let user_data = unsafe {
        Arc::increment_strong_count(user_data);
        Arc::from_raw(user_data as *mut tokio::sync::mpsc::UnboundedSender<UsbEvent>)
    };
    // Fire and forget
    let _ = user_data.send(UsbEvent::FileDescriptor(UsbFileDescriptorEventData::Add(
        fd, events,
    )));
}
extern "C" fn libusb_fds_removed(fd: std::os::raw::c_int, user_data: *mut ::std::os::raw::c_void) {
    let user_data = unsafe {
        Arc::increment_strong_count(user_data);
        Arc::from_raw(user_data as *mut tokio::sync::mpsc::UnboundedSender<UsbEvent>)
    };
    // Fire and forget
    let _ = user_data.send(UsbEvent::FileDescriptor(
        UsbFileDescriptorEventData::Remove(fd),
    ));
}
