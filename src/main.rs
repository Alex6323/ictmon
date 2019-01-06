use std::collections::VecDeque;
use std::env;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

const ADDRESS: &str = "tcp://192.168.1.5"; //change this to localhost
const PORT: u16 = 5560;
const CHANNEL_TX: &'static str = "tx";

struct Arguments {
    program: String,
    address: String,
    port: u16,
}

impl Arguments {
    pub fn new(args: Vec<String>) -> Result<Self, &'static str> {
        if args.len() == 1 {
            Ok(Arguments {
                program: args[0].clone(),
                address: ADDRESS.to_string(),
                port: PORT,
            })
        } else if args.len() == 3 {
            Ok(Arguments {
                program: args[0].clone(),
                address: args[1].clone(),
                port: args[2].parse::<u16>().unwrap(),
            })
        } else {
            Err("Wrong number of arguments provided. Usage: ixi_zmq_listener <IP> <Port>")
        }
    }
}
fn main() {
    if let Ok(args) = Arguments::new(env::args().collect::<Vec<String>>()) {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        let address = format!("{}:{}", args.address, args.port);

        subscriber
            .connect(&address)
            .expect(&format!("Could not connect to publisher '{}'.", address));

        println!("Connected to address '{}'.", address);

        let subscription = CHANNEL_TX.to_string().into_bytes();
        subscriber.set_subscribe(&subscription).unwrap();

        println!("Subscribed to channel: '{:?}'", CHANNEL_TX);

        loop {
            subscriber.recv_msg(0).unwrap();
        }
    } else {
        process::exit(0);
    }
}
