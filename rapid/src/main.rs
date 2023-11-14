use log::{debug, error};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use serde::Deserialize;
use std::{mem::size_of, net::UdpSocket, thread};

const BIND_ADDR: &str = "0.0.0.0:12345";
const BUF_SIZE: usize = size_of::<Data>();
const REQ_REGISTER: i32 = 1;
const REQ_UNREGISTER: i32 = 0;
const REQ_STOP: i32 = 2;
const REQ_CONT: i32 = 3;

#[derive(Deserialize, Debug)]
struct Data {
    req: i32,
    pid: i32,
}

fn main() -> Result<(), ()> {
    env_logger::init();
    let mut queue: Vec<i32> = vec![];
    let stream = UdpSocket::bind(BIND_ADDR).unwrap();
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        stream.recv(&mut buf).unwrap();
        let data: Data = bincode::deserialize_from(&buf[..]).unwrap();
        debug!("received_msg: {:?}", data);
        match data.req {
            REQ_REGISTER => {
                debug!("register_process: {:?}", data.pid);
                queue.push(data.pid);
            }
            REQ_UNREGISTER => {
                debug!("unregister_process: {:?}", data.pid);
                let pos = queue.iter().position(|e| *e == data.pid).unwrap();
                queue.remove(pos);
            }
            REQ_STOP => {
                debug!("send_signal: SIGSTOP");
                let signal = Signal::SIGSTOP;
                send_signal(&queue, signal).unwrap();
            }
            REQ_CONT => {
                debug!("send_signal: SIGCONT");
                let signal = Signal::SIGCONT;
                send_signal(&queue, signal).unwrap();
            }
            _ => {
                error!("unlnown request received: {:?}", data);
                return Err(());
            }
        };
    }
    Ok(())
}

fn send_signal(targets: &[i32], signal: Signal) -> Result<(), ()> {
    for pid in targets.iter() {
        let target = *pid;
        thread::spawn(move || kill(Pid::from_raw(target), signal).unwrap());
    }
    Ok(())
}
