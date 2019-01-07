use std::cmp::min;
use std::collections::VecDeque;
use std::env;
use std::io::{self, Write};
use std::process;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime};

const ADDRESS: &str = "192.168.1.5"; //change this to localhost
const PORT: u16 = 5560;
const CHANNEL_TX: &'static str = "tx";
const MOVING_AVG_INTERVAL_MS: u64 = 60000;
const VERSION: &'static str = "v0.1";

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
    println!("Welcome to Ict Network Monitor {}", VERSION);

    let args: Arguments;
    match Arguments::new(env::args().collect::<Vec<String>>()) {
        Ok(a) => args = a,
        Err(s) => {
            println!("{}", s);
            process::exit(0);
        }
    }

    let context = zmq::Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    let address = format!("tcp://{}:{}", args.address, args.port);

    subscriber.connect(&address).expect(&format!(
        "Could not connect to publisher: '{}:{}'.",
        args.address, args.port
    ));

    println!(
        "Successfully established connection to zeromq IXI running at '{}:{}'",
        args.address, args.port
    );

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

    thread::sleep(Duration::from_millis(1000));

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
        thread::sleep(Duration::from_millis(1000));
    }

    fn print_tps(tps: f64) {
        print!(
            "+--------------+\n|{:>10.2} tps|\n+--------------+\r\x1b[2A",
            tps
        );
        io::stdout().flush().unwrap();
    }
}
