use serde::{Deserialize, Serialize};
use std::ffi::c_int;

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub req: ReqType,
    pub pid: c_int,
}

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub enum ReqType {
    Unregister = 0,
    Register = 1,
    Stop = 2,
    Cont = 3,
    CommBegin = 4,
    CommEnd = 5,
}
