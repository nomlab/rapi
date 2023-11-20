use clap::Parser;
use log::debug;
use serde::{Deserialize, Serialize};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{mem::size_of, net::UdpSocket, str::FromStr, thread, thread::sleep, time::Duration};

const DEFAULT_PORT: u16 = 54321;
const DEFAULT_TPORT: u16 = 12345;
const DEFAULT_DLEVEL: &str = "Error";

const BUF_SIZE: usize = size_of::<Data>();
const BIND_ADDR: &str = "0.0.0.0";

const REQ_STOP: i32 = 2;
const REQ_CONT: i32 = 3;

const FIRST_REQ: Data = Data {
    req: REQ_STOP,
    dummy: 0,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Duration (ms) between suspending and resuming job.
    /// If timeslice < 0, turn off job switching.
    #[arg(short, long, required = true)]
    timeslice: i64,

    /// The list of nodes to communicate with rapid.
    /// Example: "-n localhost", "-n com1,com2", "-n com1 -n com2"
    #[arg(short, long, required = true, value_delimiter = ',')]
    nodes: Vec<String>,

    /// Port to bind.
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Target port to send request.
    #[arg(long, default_value_t = DEFAULT_TPORT)]
    tport: u16,

    /// Debug level.
    /// One of [Error, Warn, Info, Debug, Trace, Off].
    #[arg(short, long, default_value_t = String::from(DEFAULT_DLEVEL))]
    debug: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    req: i32,
    dummy: i32,
}

fn main() {
    let args = Args::parse();
    SimpleLogger::init(
        LevelFilter::from_str(&args.debug).unwrap(),
        Config::default(),
    )
    .unwrap();

    let socket = UdpSocket::bind((BIND_ADDR, args.port)).unwrap();
    let sender_socket = socket.try_clone().unwrap();

    if args.timeslice > 0 {
        let duration = Duration::from_millis(args.timeslice.try_into().unwrap());
        thread::spawn(move || {
            send_req(sender_socket, duration, &args.nodes, args.tport).unwrap();
        });
        recv_req(socket).unwrap();
    } else {
        recv_req(socket).unwrap();
    }
}

fn send_req(
    stream: UdpSocket,
    duration: Duration,
    nodes: &[String],
    tport: u16,
) -> Result<(), std::io::Error> {
    let mut req = FIRST_REQ;
    loop {
        let buf = bincode::serialize(&req).unwrap();
        for host in nodes.iter() {
            debug!("Send request: {:?} to: {}", req, host);
            stream.send_to(&buf, (host.as_str(), tport))?;
        }
        reverse_request(&mut req).unwrap();
        sleep(duration);
    }
}

fn recv_req(stream: UdpSocket) -> Result<(), std::io::Error> {
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    loop {
        stream.recv(&mut buf)?;
        let req: Data = bincode::deserialize(&buf).unwrap();
        debug!("Receive request: {:?}", req);
    }
}

fn reverse_request(data: &mut Data) -> Result<(), ()> {
    match data.req {
        REQ_STOP => data.req = REQ_CONT,
        REQ_CONT => data.req = REQ_STOP,
        _ => return Err(()),
    }
    Ok(())
}
