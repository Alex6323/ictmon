use std::cmp::min;
use std::collections::VecDeque;
use std::env;
use std::io::{self, Write};
use std::process;
use std::time::{Duration, SystemTime};

const ADDRESS: &str = "tcp://192.168.1.5"; //change this to localhost
const PORT: u16 = 5560;
const CHANNEL_TX: &'static str = "tx";
const MOVING_AVG_INTERVAL_MS: u64 = 60000;

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

        println!("Connected to publisher: '{}'.", address);

        let subscription = CHANNEL_TX.to_string().into_bytes();
        subscriber.set_subscribe(&subscription).unwrap();

        println!("Subscribed to channel: '{}'", CHANNEL_TX);

        let mut now: Duration;
        let interval = Duration::from_millis(MOVING_AVG_INTERVAL_MS);
        let mut start: Duration;
        let mut events: VecDeque<Duration> = VecDeque::new();
        let mut uptime_ms: u64;

        let init = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        loop {
            subscriber.recv_msg(0).unwrap();
            now = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            start = now - interval;
            events.push_back(now);

            while events.front().unwrap() < &start {
                events.pop_front();
            }

            uptime_ms = (now - init).as_secs() * 1000 + (now - init).subsec_millis() as u64;
            print!(
                "\rTPS: {:.2}",
                events.len() as f64 / (min(MOVING_AVG_INTERVAL_MS, uptime_ms) as f64 / 1000_f64)
            );
            io::stdout().flush().unwrap();
        }
    } else {
        process::exit(0);
    }
}
