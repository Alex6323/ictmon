use std::{
    cmp::min,
    io::{self, Write},
    time::{Duration, Instant},
};

use tokio::{
    prelude::*,
    runtime::Runtime,
    timer::{Delay, Interval},
};

use crate::constants::*;
use crate::Arguments;
use crate::IctNode;

use zmq::Context;

pub struct Metrics(pub f32);

pub fn spawn_receiver_tasks(runtime: &mut Runtime, args: &Arguments) {
    args.nodes
        .iter()
        .for_each(|n| spawn_receiver_task(runtime, n, args.topic.clone()));
}

pub fn spawn_receiver_task(runtime: &mut Runtime, node: &IctNode, topic: String) {
    let context = Context::new();
    let subscriber = context.socket(zmq::SUB).unwrap();
    let address = format!("tcp://{}:{}", node.address, node.port);

    //TODO: get proper error message (maybe two lines are the same?)
    subscriber.connect(&address).unwrap_or_else(|_| {
        panic!(
            "Failed to connect to Ict node {} ({}:{}).",
            node.name, node.address, node.port
        )
    });

    println!(
        "Info: Listening to Ict node {} ({}:{}) ...",
        node.name, node.address, node.port
    );

    let subscription = topic.as_bytes();
    subscriber.set_subscribe(&subscription).unwrap();

    let arrivals_move = node.arrivals.clone();
    let receiver_task = Delay::new(Instant::now())
        .and_then(move |_| {
            let mut arrival_timestamp: Instant;
            loop {
                // For now, we are not interested in the message itself
                subscriber.recv_msg(0).unwrap();
                arrival_timestamp = Instant::now();

                let mut queue = arrivals_move.lock().unwrap();
                queue.push_back(arrival_timestamp);
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in receiver task: {:?}", e));

    runtime.spawn(receiver_task);
}

pub fn spawn_tps_tasks(runtime: &mut Runtime, args: &Arguments) {
    args.nodes.iter().for_each(|n| spawn_tps_task(runtime, n));
}

pub fn spawn_tps_task<'a>(runtime: &mut Runtime, node: &IctNode) {
    let interval = Duration::from_secs(MOVING_AVG_INTERVAL_SEC);

    let mut uptime_sec: u64 = 0;
    let init = Instant::now();

    let arrivals_move = node.arrivals.clone();
    let metrics_move = node.metrics.clone();
    let tps_task = Interval::new_interval(Duration::from_millis(UPDATE_INTERVAL_MS))
        .for_each(move |instant| {
            let window_start = instant - interval;
            {
                let mut queue = arrivals_move.lock().unwrap();
                while queue.len() > 0 && queue.front().unwrap() < &window_start {
                    queue.pop_front();
                }

                uptime_sec = (instant - init).as_secs();
                {
                    metrics_move.lock().unwrap().0 =
                        queue.len() as f32 / (min(MOVING_AVG_INTERVAL_SEC, uptime_sec) as f32);
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in tps task: {:?}", e));

    runtime.spawn(tps_task);
}
use std::sync::{Arc, Mutex};

pub fn spawn_stdout_task(runtime: &mut Runtime, args: &Arguments) {
    let mut m: Vec<Arc<Mutex<Metrics>>> = vec![];
    args.nodes.iter().for_each(|n| m.push(n.metrics.clone()));

    let stdout_task = Interval::new_interval(Duration::from_millis(STDOUT_UPDATE_INTERVAL_MS))
        .for_each(move |_| {
            m.iter().for_each(|n| {
                print!("{:.2} tps   ", n.lock().unwrap().0);
            });
            print!("\r");
            //{
            //print!(
            //    "\r\x1b[2A+--------------+\n|{:>10.2} tps |\n+--------------+",
            //    metrics.lock().unwrap().0
            //);
            //}
            io::stdout().flush().unwrap();
            Ok(())
        })
        .map_err(|e| panic!("Error in stdout task: {:?}", e));

    runtime.spawn(stdout_task);
}

pub fn spawn_responder_task(runtime: &mut Runtime, args: &Arguments) {
    let responder_context = Context::new();
    let responder = responder_context
        .socket(zmq::REP)
        .expect("Failed to create responder from ZMQ context.");

    let address = format!("tcp://*:{}", DEFAULT_API_PORT);
    responder
        .bind(&address)
        .expect("Could not bind responder socket.");

    let metrics_move = args.nodes[0].metrics.clone();
    let responder_task = Delay::new(Instant::now())
        .and_then(move |_| {
            loop {
                responder.recv_string(0).unwrap().unwrap();
                //println!("Received request.");
                {
                    responder
                        .send(&format!("{:.2}", metrics_move.lock().unwrap().0), 0)
                        .unwrap();
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in responder task: {:?}", e));

    runtime.spawn(responder_task);
}
