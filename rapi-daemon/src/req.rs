use serde::{Deserialize, Serialize};

#[repr(C)]
#[derive(Serialize, Deserialize, Debug)]
pub struct Data {
    pub req: ReqType,
    pub pid: i32,
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
