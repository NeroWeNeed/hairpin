use std::{
    borrow::Cow, collections::BTreeMap, marker::PhantomData, path::Path, sync::Arc, time::Duration,
};

use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;

use crate::{
    error::{Error, ServeError},
    event::{Event, MountEvent, MountEventMask},
    monitor::Monitor,
    table::Table,
    util::{get_fstab_path, get_mtab_path, get_utab_path},
};
pub trait Handler<'a> {
    type Error: std::error::Error;
    fn handle(&self, evt: MountEvent<'a>) -> Result<(), Self::Error>;
}
pub fn handler<'a, F, E>(value: F) -> HandlerFn<F, E>
where
    F: Fn(MountEvent<'a>) -> Result<(), E>,
    E: std::error::Error,
{
    HandlerFn(value, PhantomData::default())
}
pub struct HandlerFn<F, E>(F, PhantomData<E>);
impl<'a, F, E> From<F> for HandlerFn<F, E>
where
    F: Fn(MountEvent<'a>) -> Result<(), E>,
    E: std::error::Error,
{
    fn from(value: F) -> Self {
        Self(value, PhantomData::default())
    }
}
impl<'a, F, E> Handler<'a> for HandlerFn<F, E>
where
    F: Fn(MountEvent<'a>) -> Result<(), E>,
    E: std::error::Error,
{
    fn handle(&self, evt: MountEvent<'a>) -> Result<(), Self::Error> {
        self.0(evt)
    }

    type Error = E;
}
pub struct MonitorServe;

impl MonitorServe {
    pub fn builder<'a>() -> MonitorServeBuilderNoHandler<'a> {
        MonitorServeBuilderNoHandler::default()
    }
}
#[derive(Default, Clone)]
pub struct MonitorServeBuilderNoHandler<'a> {
    userspace: Option<(bool, Option<&'a Path>)>,
    kernel: Option<bool>,
    poll_rate: Option<Duration>,
}
impl<'a> MonitorServeBuilderNoHandler<'a> {
    pub fn with_handler<E>(
        self,
        mask: impl Into<MountEventMask>,
        handler: impl Handler<'a, Error = E> + Send + Sync + 'a,
    ) -> MonitorServeBuilder<'a, E>
    where
        E: std::error::Error,
    {
        let handlers: Vec<(_, Arc<Box<dyn Handler<'a, Error = E> + Send + Sync + 'a>>)> =
            vec![(mask.into(), Arc::new(Box::new(handler)))];
        MonitorServeBuilder {
            handlers,
            userspace: self.userspace,
            kernel: self.kernel,
            poll_rate: self.poll_rate,
            err: PhantomData::default(),
        }
    }
    pub fn with_userspace(mut self, value: bool, file: Option<&'a Path>) -> Self {
        self.userspace = Some((value, file));
        self
    }
    pub fn with_kernel(mut self, value: bool) -> Self {
        self.kernel = Some(value);
        self
    }
    pub fn with_poll_rate(mut self, value: Duration) -> Self {
        self.poll_rate = Some(value);
        self
    }
}
#[derive(Clone)]
pub struct MonitorServeBuilder<'a, E> {
    handlers: Vec<(
        MountEventMask,
        Arc<Box<dyn Handler<'a, Error = E> + Send + Sync + 'a>>,
    )>,
    userspace: Option<(bool, Option<&'a Path>)>,
    kernel: Option<bool>,
    poll_rate: Option<Duration>,
    err: PhantomData<E>,
}
impl<'a, E> Default for MonitorServeBuilder<'a, E> {
    fn default() -> Self {
        Self {
            handlers: Vec::default(),
            userspace: Option::default(),
            kernel: Option::default(),
            poll_rate: Option::default(),
            err: PhantomData::default(),
        }
    }
}
impl<'a, E> MonitorServeBuilder<'a, E>
where
    E: std::error::Error,
{
    pub fn with_handler(
        mut self,
        mask: impl Into<MountEventMask>,
        handler: impl Handler<'a, Error = E> + Send + Sync + 'a,
    ) -> Self {
        self.handlers
            .push((mask.into(), Arc::new(Box::new(handler))));
        self
    }

    pub fn with_userspace(mut self, value: bool, file: Option<&'a Path>) -> Self {
        self.userspace = Some((value, file));
        self
    }
    pub fn with_kernel(mut self, value: bool) -> Self {
        self.kernel = Some(value);
        self
    }
    pub fn with_poll_rate(mut self, value: Duration) -> Self {
        self.poll_rate = Some(value);
        self
    }
}
impl<E> MonitorServeBuilder<'static, E>
where
    E: std::error::Error + Send + Sync + 'static,
{
    pub fn build(
        self,
    ) -> Result<
        (
            impl Future<Output = Result<(), ServeError<E>>> + 'static,
            tokio::sync::oneshot::Sender<()>,
            tokio::sync::mpsc::UnboundedReceiver<ServeError<E>>,
        ),
        ServeError<E>,
    > {
        fn create_event_handler<E>(
            handlers: Vec<(
                MountEventMask,
                Arc<Box<dyn Handler<'static, Error = E> + Send + Sync + 'static>>,
            )>,
        ) -> (
            impl Future<Output = Result<(), ServeError<E>>> + 'static,
            UnboundedSender<Event<'static>>,
            tokio::sync::mpsc::UnboundedReceiver<ServeError<E>>,
        )
        where
            E: std::error::Error + Send + Sync + 'static,
        {
            let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
            let (error_tx, error_rx) = tokio::sync::mpsc::unbounded_channel();
            (
                async move {
                    let mut receiver = event_rx;
                    while let Some(evt) = receiver.recv().await {
                        match evt {
                            Event::MountEvent(evt) => {
                                for (mask, handler) in &handlers {
                                    let evt = evt.clone();
                                    let error_tx = error_tx.clone();
                                    if mask.matches(&evt) {
                                        let handler = handler.clone();
                                        tokio::spawn(async move {
                                            if let Err(err) = handler.handle(evt) {
                                                let _ = error_tx.send(ServeError::Handler(err));
                                            }
                                        });
                                    }
                                }
                            }
                            Event::Close => {
                                receiver.close();
                            }
                            Event::Error(error) => {
                                let _ = error_tx.send(ServeError::LibMount(error));
                            }
                        }
                    }
                    Ok(())
                },
                event_tx,
                error_rx,
            )
        }
        fn create_poll_handler<E>(
            monitor: Monitor,
            event_sender: tokio::sync::mpsc::UnboundedSender<Event<'static>>,
            poll_rate: Duration,
            fstab: bool,
            mtab: bool,
        ) -> (
            impl Future<Output = Result<(), ServeError<E>>> + 'static,
            tokio::sync::oneshot::Sender<()>,
        )
        where
            E: std::error::Error,
        {
            let (tx, rx) = tokio::sync::oneshot::channel();
            (
                async move {
                    let mut rx = rx;
                    let event_sender_poll = event_sender.clone();
                    let mut initial = BTreeMap::<Cow<'_, Path>, Table>::new();
                    if fstab {
                        initial.insert(
                            get_fstab_path(),
                            Table::parse_fstab(None).map_err(Error::AllocationTable)?,
                        );
                    }
                    if mtab {
                        initial.insert(
                            get_mtab_path(),
                            Table::parse_mtab(None).map_err(Error::AllocationTable)?,
                        );
                    }
                    monitor
                        .poll_until(
                            poll_rate,
                            initial,
                            true,
                            move || match rx.try_recv() {
                                Ok(_) => true,
                                Err(err) => match err {
                                    tokio::sync::oneshot::error::TryRecvError::Empty => false,
                                    tokio::sync::oneshot::error::TryRecvError::Closed => true,
                                },
                            },
                            |evt| event_sender_poll.send(evt).is_ok(),
                        )
                        .await?;
                    let _ = event_sender.send(Event::Close);
                    Ok(())
                },
                tx,
            )
        }
        let mut monitor = Monitor::new().map_err(Error::AllocationMonitor)?;
        if let Some(kernel) = self.kernel {
            monitor.with_kernel(kernel)?;
        };
        if let Some((userspace, file)) = self.userspace {
            monitor.with_userspace(userspace, file)?;
        }
        let (event_future, event_sender, error_receiver) = create_event_handler(self.handlers);
        let (poll_future, close_signal) = create_poll_handler(
            monitor,
            event_sender,
            self.poll_rate.unwrap_or(Duration::ZERO),
            true,
            true,
        );
        Ok((
            async move {
                let event_handle = tokio::spawn(event_future);
                let poll_result = poll_future.await;
                let event_result = event_handle.await;
                poll_result?;
                event_result??;
                Ok(())
            },
            close_signal,
            error_receiver,
        ))
    }
    pub async fn serve(self) -> Result<(), ServeError<E>> {
        let (future, close, errors) = self.build()?;
        let ctrlc_cancel = CancellationToken::new();
        let ctrlc_cancel_1 = ctrlc_cancel.clone();
        let ctrlc_handler = tokio::spawn(async move {
            let mut errors = errors;
            tokio::select! {
                io = tokio::signal::ctrl_c() => {
                    match io {
                        Ok(_) => { let _ = close.send(()); },
                        err => { return err }
                    }
                }
                _ = ctrlc_cancel_1.cancelled() => { }
                _ = errors.recv() => { let _ = close.send(()); }
            }
            Ok::<_, std::io::Error>(())
        });

        let result_1 = future.await;
        ctrlc_cancel.cancel();
        let result_2 = ctrlc_handler.await;
        result_1?;
        result_2??;
        Ok::<(), ServeError<E>>(())
    }
}
