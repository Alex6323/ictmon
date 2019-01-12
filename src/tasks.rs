use std::{
    cmp::min,
    collections::VecDeque,
    io::{self, Write},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio;
use tokio::runtime::Runtime;
use tokio::{prelude::*, timer::Delay, timer::Interval};

use crate::Arguments;
use lazy_static::lazy_static;

use zmq::{Context, Socket};
const ZMQ_PORT: u16 = 5560;

const PRINT_UPDATE_INTERVAL_MS: u64 = 1000;
const UPDATE_INTERVAL_MS: u64 = 1000;

pub struct Metrics(pub f64);

const MOVING_AVG_INTERVAL_MS: u64 = 60000;

lazy_static! {
    static ref CHANNEL_TX: String = String::from("tx");
}

pub fn spawn_receiver_task(
    runtime: &mut Runtime,
    arrival_timestamps: Arc<Mutex<VecDeque<Instant>>>,
    args: &Arguments,
) {
    let context = Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    let address = format!("tcp://{}:{}", args.address, args.port);

    subscriber.connect(&address).unwrap_or_else(|_| {
        panic!(
            "Fatal: Could not connect to publisher: '{}:{}'.",
            args.address, args.port
        )
    });

    println!(
        "Info: Connected to Ict node running ZeroMQ IXI extension module at '{}:{}'.",
        args.address, args.port
    );

    let subscription = CHANNEL_TX.as_bytes();
    subscriber.set_subscribe(&subscription).unwrap();

    let arrival_timestamps_recv = arrival_timestamps.clone();

    let receiver_task = Delay::new(Instant::now())
        .and_then(move |_| {
            let mut arrival_timestamp: Instant;
            loop {
                subscriber.recv_msg(0).unwrap();
                arrival_timestamp = Instant::now();

                let mut queue = arrival_timestamps_recv.lock().unwrap();
                queue.push_back(arrival_timestamp);
            }
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    runtime.spawn(receiver_task);
}

pub fn spawn_tps_task(
    runtime: &mut Runtime,
    arrival_timestamps: Arc<Mutex<VecDeque<Instant>>>,
    metrics: Arc<Mutex<Metrics>>,
) {
    let interval = Duration::from_millis(MOVING_AVG_INTERVAL_MS);

    let mut uptime_ms: u64 = 0;
    let init = Instant::now();

    let tps_task = Interval::new_interval(Duration::from_millis(UPDATE_INTERVAL_MS))
        .for_each(move |instant| {
            let window_start = instant - interval;
            {
                let mut queue = arrival_timestamps.lock().unwrap();
                while queue.len() > 0 && queue.front().unwrap() < &window_start {
                    queue.pop_front();
                }
                uptime_ms =
                    (instant - init).as_secs() * 1000 + u64::from((instant - init).subsec_millis());
                {
                    metrics.lock().unwrap().0 = queue.len() as f64
                        / (min(MOVING_AVG_INTERVAL_MS, uptime_ms) as f64 / 1000_f64);
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("interval errored; err={:?}", e));

    runtime.spawn(tps_task);
}

pub fn spawn_stdout_task(runtime: &mut Runtime, metrics: Arc<Mutex<Metrics>>) {
    let stdout_task = Interval::new_interval(Duration::from_millis(PRINT_UPDATE_INTERVAL_MS))
        .for_each(move |_| {
            {
                print!(
                    "\r\x1b[2A+--------------+\n|{:>10.2} tps|\n+--------------+",
                    metrics.lock().unwrap().0
                );
            }
            io::stdout().flush().unwrap();
            Ok(())
        })
        .map_err(|e| panic!("Couldn't create stdout task: err={:?}", e));

    runtime.spawn(stdout_task);
}

pub fn spawn_responder_task(runtime: &mut Runtime, metrics: Arc<Mutex<Metrics>>) {
    let responder_context = Context::new();
    let responder = responder_context
        .socket(zmq::REP)
        .expect("Failed to create respond from ZMQ context.");

    let address = format!("tcp://*:{}", 5560);
    responder
        .bind(&address)
        .expect("Could not bind responder socket.");

    let responder_task = Delay::new(Instant::now())
        .and_then(move |_| {
            loop {
                responder.recv_string(0).unwrap().unwrap();
                {
                    responder
                        .send(&format!("{:.2}", metrics.lock().unwrap().0), 0)
                        .unwrap();
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("error in response task: {:?}", e));

    runtime.spawn(responder_task);
}
