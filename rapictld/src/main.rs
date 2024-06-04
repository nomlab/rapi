use clap::Parser;
use log::debug;
use serde::{Deserialize, Serialize};
use simplelog::{Config, LevelFilter, SimpleLogger};
use std::{
    mem::size_of,
    net::UdpSocket,
    str::FromStr,
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    thread,
    thread::sleep,
    time::{Duration, Instant},
};

const TIMESLICE_IN_COMM: Duration = Duration::from_millis(100);
const TIMESLICE_GUARANTEED: Duration = Duration::from_millis(400);
const TIMESLICE_CHECK_INTERVAL: Duration = Duration::from_millis(1);

const DEFAULT_PORT: u16 = 8211;
const DEFAULT_RAPID_PORT: u16 = 8210;
const DEFAULT_DLEVEL: &str = "Error";

const BUF_SIZE: usize = size_of::<Data>();
const BIND_ADDR: &str = "0.0.0.0";

const REQ_UNREGISTER: i32 = 0;
const REQ_REGISTER: i32 = 1;
const REQ_STOP: i32 = 2;
const REQ_CONT: i32 = 3;
const REQ_BEGIN_COMM: i32 = 4;
const REQ_END_COMM: i32 = 5;

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
    _timeslice: i64,

    /// The list of nodes to communicate with rapid.
    /// Example: "-n localhost", "-n com1,com2", "-n com1 -n com2"
    #[arg(short, long, required = true, value_delimiter = ',')]
    nodes: Vec<String>,

    /// Port to bind.
    #[arg(long, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Target port to send request.
    #[arg(long, default_value_t = DEFAULT_RAPID_PORT)]
    rapid_port: u16,

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
    let count_in_communication = Arc::new(AtomicU32::new(0));
    SimpleLogger::init(
        LevelFilter::from_str(&args.debug).unwrap(),
        Config::default(),
    )
    .unwrap();

    let socket = UdpSocket::bind((BIND_ADDR, args.port)).unwrap();
    let sender_socket = socket.try_clone().unwrap();
    {
        let count_in_communication = Arc::clone(&count_in_communication);
        thread::spawn(move || {
            recv_req_loop(socket, &count_in_communication).unwrap();
        });
    }

    let mut is_job_running = true;
    let mut instant = Instant::now();
    loop {
        let elapsed = instant.elapsed();

        if is_job_running {
            if elapsed >= TIMESLICE_GUARANTEED
                || count_in_communication.load(Ordering::Relaxed) > 0
                    && elapsed >= TIMESLICE_IN_COMM
            {
                let stop_req = Data {
                    req: REQ_STOP,
                    dummy: 0,
                };
                send_req(&sender_socket, &stop_req, &args.nodes, args.rapid_port).unwrap();
                is_job_running = false;
                instant = Instant::now();
            }
        } else {
            #[warn(clippy::collapsible_else_if)]
            if elapsed >= TIMESLICE_IN_COMM {
                let cont_req = Data {
                    req: REQ_CONT,
                    dummy: 0,
                };
                send_req(&sender_socket, &cont_req, &args.nodes, args.rapid_port).unwrap();
                is_job_running = true;
                instant = Instant::now();
            }
        }
        sleep(TIMESLICE_CHECK_INTERVAL);
    }
}

fn send_req(
    socket: &UdpSocket,
    req: &Data,
    nodes: &[String],
    tport: u16,
) -> Result<(), std::io::Error> {
    let buf = bincode::serialize(&req).unwrap();
    for host in nodes.iter() {
        debug!("Send request: {:?} to: {}", req, host);
        socket.send_to(&buf, (host.as_str(), tport))?;
    }
    Ok(())
}

fn recv_req_loop(
    socket: UdpSocket,
    count_in_communication: &AtomicU32,
) -> Result<(), std::io::Error> {
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    loop {
        socket.recv(&mut buf)?;
        let req: Data = bincode::deserialize(&buf).unwrap();
        match req.req {
            REQ_BEGIN_COMM => {
                count_in_communication.fetch_add(1, Ordering::Relaxed);
            }
            REQ_END_COMM => {
                count_in_communication.fetch_sub(1, Ordering::Relaxed);
            }
            _ => {}
        };
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
