use clap::Parser;
use log::debug;
use nix::{
    sys::signal::{kill, Signal},
    unistd::Pid,
};
use rapi::req::{Data, ReqType};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{mem::size_of, net::UdpSocket, str::FromStr, thread};

const DEFAULT_PORT: u16 = 8210;
const DEFAULT_RAPICTLD_PORT: u16 = 8211;
const DEFAULT_DLEVEL: &str = "Error";

const BIND_ADDR: &str = "0.0.0.0";
const BUF_SIZE: usize = size_of::<Data>();

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Address of rapictld (IP Address (192.168.1.2) or Dmain (rapictld.example.com))
    #[arg(short = 'a', long, required = true)]
    rapictld_addr: String,

    /// Port of rapictld
    #[arg(short = 'P', long, default_value_t = DEFAULT_RAPICTLD_PORT)]
    rapictld_port: u16,

    /// Port to bind
    #[arg(short = 'p', long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Debug level (One of [Error, Warn, Info, Debug, Trace, Off])
    #[arg(short = 'd', long, default_value_t = String::from(DEFAULT_DLEVEL))]
    debug: String,
}

impl Args {
    pub fn rapictld_socket_addr(&self) -> (&str, u16) {
        (&self.rapictld_addr, self.rapictld_port)
    }
}

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
        let data: Data = bincode::deserialize(&buf).unwrap();
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
