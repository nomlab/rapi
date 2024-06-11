mod args;

use args::Args;
use clap::Parser;
use log::debug;
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use rapi::{
    req::{ReqType, Request},
    *, // import some consts
};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{mem::size_of, net::UdpSocket, str::FromStr, thread};

const BUF_SIZE: usize = size_of::<Request>();

fn main() -> Result<(), ()> {
    let args = Args::parse();
    SimpleLogger::init(
        LevelFilter::from_str(&args.debug).unwrap(),
        Config::default(),
    )
    .unwrap();

    let mut queue: Vec<i32> = vec![];
    let stream = UdpSocket::bind((BIND_ADDR, args.port)).unwrap();
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        stream.recv(&mut buf).unwrap();
        let data: Request = bincode::deserialize(&buf).unwrap();
        debug!("Recv request: {:?}", data);

        stream.send_to(&buf, args.rapictld_socket_addr()).unwrap();
        debug!("Send request: {:?}", data);

        match data.req {
            ReqType::Register => {
                queue.push(data.pid);
            }
            ReqType::Unregister => {
                let pos = queue.iter().position(|e| *e == data.pid).unwrap();
                queue.remove(pos);
            }
            ReqType::Stop => {
                let signal = Signal::SIGSTOP;
                send_signal(&queue, signal).unwrap();
            }
            ReqType::Cont => {
                let signal = Signal::SIGCONT;
                send_signal(&queue, signal).unwrap();
            }
            ReqType::CommBegin | ReqType::CommEnd => {}
        };
    }
}

fn send_signal(targets: &[i32], signal: Signal) -> Result<(), ()> {
    for pid in targets.iter() {
        let target = *pid;
        thread::spawn(move || kill(Pid::from_raw(target), signal).unwrap());
    }
    Ok(())
}
