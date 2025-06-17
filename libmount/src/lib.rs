pub(crate) mod libmount {
    #![allow(non_upper_case_globals)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(unsafe_op_in_unsafe_fn)]
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
}

pub mod context;
pub mod error;
pub mod event;
pub mod fs;
pub mod iter;
pub mod monitor;
pub mod serve;
pub mod table;
pub mod update;
pub(crate) mod util;
