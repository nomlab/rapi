use log::{debug, error};
use serde::Serialize;
use std::{env, net::UdpSocket, thread::sleep, time::Duration};

const BIND_ADDR: &str = "0.0.0.0:0";
const PORT: u16 = 12345;
const REQ_STOP: i32 = 2;
const REQ_CONT: i32 = 3;

#[derive(Serialize, Debug)]
struct Data {
    req: i32,
    dummy: i32,
}

fn main() -> Result<(), ()> {
    env_logger::init();
    let args: Vec<String> = env::args().collect();
    if args.get(1).is_none() || args.get(2).is_none() {
        error!("TimeSlice or Hostname is missing");
        error!("{} <TimeSlice (msec)> <Host1> (<Host2> ..)", args[0]);
        return Err(());
    }
    let time_slice: u64 = args[1].parse().unwrap();
    let hosts: Vec<&str> = args[2..].iter().map(|arg| arg.as_str()).collect();
    let mut req = Data {
        req: REQ_STOP,
        dummy: 0,
    };
    loop {
        let buf = bincode::serialize(&req).unwrap();
        let stream = UdpSocket::bind(BIND_ADDR).unwrap();
        for host in hosts.iter() {
            debug!("send_to: {}, msg: {:?}", host, req);
            stream.send_to(&buf, (*host, PORT)).unwrap();
        }
        reverse_request(&mut req).unwrap();
        sleep(Duration::from_millis(time_slice));
    }

    Ok(())
}

fn reverse_request(data: &mut Data) -> Result<(), ()> {
    match data.req {
        REQ_STOP => data.req = REQ_CONT,
        REQ_CONT => data.req = REQ_STOP,
        _ => return Err(()),
    }
    Ok(())
}
