use clap::Parser;
use rapi::*; // import some consts

#[derive(Parser, Debug)]
#[command(author, version, about)]
pub struct Args {
    /// Address of rapictld (IP Address (192.168.1.2) or Dmain (rapictld.example.com))
    #[arg(short = 'a', long, required = true)]
    pub rapictld_addr: String,

    /// Port of rapictld
    #[arg(short = 'P', long, default_value_t = DEFAULT_RAPICTLD_PORT)]
    pub rapictld_port: u16,

    /// Port to bind
    #[arg(short = 'p', long, default_value_t = DEFAULT_RAPID_PORT)]
    pub port: u16,

    /// Debug level (One of [Error, Warn, Info, Debug, Trace, Off])
    #[arg(short = 'd', long, default_value_t = String::from(DEFAULT_DLEVEL))]
    pub debug: String,
}

impl Args {
    pub fn rapictld_socket_addr(&self) -> (&str, u16) {
        (&self.rapictld_addr, self.rapictld_port)
    }
}
