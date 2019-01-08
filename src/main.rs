use std::cmp::min;
use std::collections::VecDeque;
use std::env;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

const VERSION: &'static str = "v0.1.0";
const NAME: &'static str = "ictmon";
const ADDRESS: &'static str = "localhost";
const PORT: u16 = 5560;
const CHANNEL_TX: &'static str = "tx";
const MOVING_AVG_INTERVAL_MS: u64 = 60000;
const INITIAL_SLEEP_MS: u64 = 1000;
const UPDATE_INTERVAL_MS: u64 = 1000;

struct Arguments {
    address: String,
    port: u16,
}

impl Arguments {
    pub fn new(args: Vec<String>) -> Result<Self, String> {
        if args.len() == 1 {
            Ok(Arguments {
                address: ADDRESS.to_string(),
                port: PORT,
            })
        } else if args.len() == 3 {
            Ok(Arguments {
                address: args[1].clone(),
                port: args[2].parse::<u16>().unwrap(),
            })
        } else {
            Err(format!(
                "Wrong number of arguments provided. Usage: ./{} <IP> <ZMQ-Port>",
                NAME
            ))
        }
    }
}

fn main() {
    let args: Arguments;
    match Arguments::new(env::args().collect::<Vec<String>>()) {
        Ok(a) => args = a,
        Err(s) => {
            println!("{}", s);
            process::exit(0);
        }
    }

    println!("Welcome to '{}' (Ict Network Monitor) {}", NAME, VERSION);

    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    let address = format!("tcp://{}:{}", args.address, args.port);

    subscriber.connect(&address).expect(&format!(
        "Could not connect to publisher: '{}:{}'.",
        args.address, args.port
    ));

    println!(
        "Info: Subscribed to Ict node running ZeroMQ IXI extension module at '{}:{}'.",
        args.address, args.port
    );

    println!("\n");

    let subscription = CHANNEL_TX.to_string().into_bytes();
    subscriber.set_subscribe(&subscription).unwrap();

    let arrival_timestamps: Arc<Mutex<VecDeque<Duration>>> = Arc::new(Mutex::new(VecDeque::new()));
    let arrival_timestamps_recv = Arc::clone(&arrival_timestamps);

    thread::spawn(move || {
        let mut arrival_timestamp: Duration;
        loop {
            subscriber.recv_msg(0).unwrap();
            arrival_timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap();

            let mut queue = arrival_timestamps_recv.lock().unwrap();
            queue.push_back(arrival_timestamp);
        }
    });

    let interval = Duration::from_millis(MOVING_AVG_INTERVAL_MS);
    let mut window_start: Duration;
    let mut now: Duration;
    let mut uptime_ms: u64;

    let init = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    thread::sleep(Duration::from_millis(INITIAL_SLEEP_MS));

    loop {
        now = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap();

        window_start = now - interval;
        {
            let mut queue = arrival_timestamps.lock().unwrap();
            while queue.len() > 0 && queue.front().unwrap() < &window_start {
                queue.pop_front();
            }
            uptime_ms = (now - init).as_secs() * 1000 + (now - init).subsec_millis() as u64;
            print_tps(
                queue.len() as f64 / (min(MOVING_AVG_INTERVAL_MS, uptime_ms) as f64 / 1000_f64),
            );
        }
        thread::sleep(Duration::from_millis(UPDATE_INTERVAL_MS));
    }

    fn print_tps(tps: f64) {
        print!(
            "\r\x1b[2A+--------------+\n|{:>10.2} tps|\n+--------------+",
            tps
        );
        io::stdout().flush().unwrap();
    }
}
