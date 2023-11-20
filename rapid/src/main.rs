use clap::Parser;
use log::{debug, error};
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use serde::{Deserialize, Serialize};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{mem::size_of, net::UdpSocket, str::FromStr, thread};

const DEFAULT_PORT: u16 = 12345;
const DEFAULT_DLEVEL: &str = "Error";

const BIND_ADDR: &str = "0.0.0.0";
const BUF_SIZE: usize = size_of::<Data>();

const REQ_REGISTER: i32 = 1;
const REQ_UNREGISTER: i32 = 0;
const REQ_STOP: i32 = 2;
const REQ_CONT: i32 = 3;

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Address of rapictld.
    #[arg(short, long, required = true)]
    addr: String,

    /// Port to bind.
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Debug level.
    /// One of [Error, Warn, Info, Debug, Trace, Off].
    #[arg(short, long, default_value_t = String::from(DEFAULT_DLEVEL))]
    debug: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    req: i32,
    pid: i32,
}

fn main() -> Result<(), ()> {
    let args = Args::parse();
    SimpleLogger::init(
        LevelFilter::from_str(&args.debug).unwrap(),
        Config::default(),
    )
    .unwrap();

    let mut queue: Vec<i32> = vec![];
    let stream = UdpSocket::bind(BIND_ADDR).unwrap();
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];

    loop {
        stream.recv(&mut buf).unwrap();
        let data: Data = bincode::deserialize(&buf).unwrap();
        debug!("Recv request: {:?}", data);

        stream.send_to(&buf, &args.addr).unwrap();
        debug!("Send request: {:?}, to: {}", data, args.addr);

        match data.req {
            REQ_REGISTER => {
                queue.push(data.pid);
            }
            REQ_UNREGISTER => {
                let pos = queue.iter().position(|e| *e == data.pid).unwrap();
                queue.remove(pos);
            }
            REQ_STOP => {
                let signal = Signal::SIGSTOP;
                send_signal(&queue, signal).unwrap();
            }
            REQ_CONT => {
                let signal = Signal::SIGCONT;
                send_signal(&queue, signal).unwrap();
            }
            _ => {
                error!("unlnown request received: {:?}", data);
                return Err(());
            }
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
