use clap::Parser;
use log::debug;
use rapi::{
    req::{ReqType, Request},
    *, // import some consts
};
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

const BUF_SIZE: usize = size_of::<Request>();

#[allow(dead_code)]
const FIRST_REQ: Request = Request {
    req: ReqType::Stop,
    pid: 0,
};

#[derive(Parser, Debug)]
#[command(author, version, about)]
struct Args {
    /// Duration (ms) between suspending and resuming job.
    /// If timeslice < 0, turn off job switching.
    #[arg(short = 't', long, required = true)]
    _timeslice: i64,

    /// Port to bind
    #[arg(short = 'p', long, default_value_t = DEFAULT_RAPICTLD_PORT)]
    port: u16,

    /// The list of all rapid's addresses (IP address or domain).
    /// Example: "node1, node2" or "192.168.1.2, 192.168.1.3"
    #[arg(short = 'a', long, required = true, value_delimiter = ',')]
    rapid_addrs: Vec<String>,

    /// Port of rapid (All rapid's port must be same)
    #[arg(short = 'P', long, default_value_t = DEFAULT_RAPID_PORT)]
    rapid_port: u16,

    /// Debug level (One of [Error, Warn, Info, Debug, Trace, Off])
    #[arg(short = 'd', long, default_value_t = String::from(DEFAULT_DLEVEL))]
    debug: String,
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
                let stop_req = Request {
                    req: ReqType::Stop,
                    pid: 0,
                };
                send_req(
                    &sender_socket,
                    &stop_req,
                    &args.rapid_addrs,
                    args.rapid_port,
                )
                .unwrap();
                is_job_running = false;
                instant = Instant::now();
            }
        } else {
            #[warn(clippy::collapsible_else_if)]
            if elapsed >= TIMESLICE_IN_COMM {
                let cont_req = Request {
                    req: ReqType::Cont,
                    pid: 0,
                };
                send_req(
                    &sender_socket,
                    &cont_req,
                    &args.rapid_addrs,
                    args.rapid_port,
                )
                .unwrap();
                is_job_running = true;
                instant = Instant::now();
            }
        }
        sleep(TIMESLICE_CHECK_INTERVAL);
    }
}

fn send_req(
    socket: &UdpSocket,
    req: &Request,
    addrs: &[String],
    port: u16,
) -> Result<(), std::io::Error> {
    let buf = bincode::serialize(&req).unwrap();
    for addr in addrs.iter() {
        socket.send_to(&buf, (addr.as_str(), port))?;
        debug!("Send request: {:?} to: {}", req, addr);
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
        let req: Request = bincode::deserialize(&buf).unwrap();
        match req.req {
            ReqType::CommBegin => {
                count_in_communication.fetch_add(1, Ordering::Relaxed);
            }
            ReqType::CommEnd => {
                count_in_communication.fetch_sub(1, Ordering::Relaxed);
            }
            _ => {}
        };
        debug!("Receive request: {:?}", req);
    }
}

#[allow(dead_code)]
fn reverse_request(data: &mut Request) -> Result<(), ()> {
    match data.req {
        ReqType::Stop => data.req = ReqType::Cont,
        ReqType::Cont => data.req = ReqType::Stop,
        _ => return Err(()),
    }
    Ok(())
}
