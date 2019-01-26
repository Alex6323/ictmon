use std::{
    cmp::min,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use tokio::{prelude::*, runtime::Runtime, timer::Interval};

use log::*;

use crate::constants::*;
use crate::display;
use crate::models::Metrics;
use crate::plot;
use crate::Arguments;
use crate::IctNode;

use zmq::Context;

///
pub fn spawn_poller_task(runtime: &mut Runtime, args: &Arguments) {
    let mut subscribers = vec![];

    args.nodes.iter().for_each(|node| {
        let context = Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        let address = format!("tcp://{}:{}", node.address, node.port);

        //TODO: get proper error message (maybe two lines in the file are the same?)
        subscriber.connect(&address).unwrap_or_else(|_| {
            panic!(
                "Failed to connect to Ict node {} ({}:{}).",
                node.name, node.address, node.port
            )
        });

        info!(
            "Listening to Ict node {} ({}:{}) ...",
            node.name, node.address, node.port
        );

        let subscription = args.topic.as_bytes();
        subscriber.set_subscribe(&subscription).unwrap();

        subscribers.push((subscriber, node.arrivals.clone(), node.arrivals2.clone()));
    });

    let mut msg = zmq::Message::new();
    let poller_task = Interval::new_interval(Duration::from_millis(POLLER_INTERVAL_MS))
        .for_each(move |_| {
            let mut poll_items = vec![];
            subscribers.iter().for_each(|(subscriber, _, _)| {
                poll_items.push(subscriber.as_poll_item(zmq::POLLIN));
            });

            zmq::poll(&mut poll_items, 10).unwrap();

            subscribers
                .iter()
                .enumerate()
                .for_each(|(i, (subscriber, arrivals, arrivals2))| {
                    if poll_items[i].is_readable() && subscriber.recv(&mut msg, 0).is_ok() {
                        let instant = Instant::now();

                        let mut queue = arrivals.lock().unwrap();
                        queue.push_back(instant);
                        let mut queue = arrivals2.lock().unwrap();
                        queue.push_back(instant);
                    }
                });
            Ok(())
        })
        .map_err(|e| panic!("Error in poller task: {:?}", e));

    runtime.spawn(poller_task);
}

pub fn spawn_tps1_tasks(runtime: &mut Runtime, args: &Arguments) {
    args.nodes.iter().for_each(|n| spawn_tps1_task(runtime, n));
}

pub fn spawn_tps1_task<'a>(runtime: &mut Runtime, node: &IctNode) {
    let interval = Duration::from_secs(MOVING_AVG_INTERVAL1_SEC);

    let mut uptime_sec: u64 = 0;
    let init = Instant::now();

    let arrivals = node.arrivals.clone();
    let metrics = node.metrics.clone();

    let tps1_task = Interval::new_interval(Duration::from_millis(TPS_UPDATE_INTERVAL_MS))
        .for_each(move |instant| {
            let window_start = instant - interval;
            {
                let mut queue = arrivals.lock().unwrap();
                while queue.len() > 0 && queue.front().unwrap() < &window_start {
                    queue.pop_front();
                }

                uptime_sec = (instant - init).as_secs();
                {
                    metrics.lock().unwrap().tps_avg1 =
                        queue.len() as f32 / (min(MOVING_AVG_INTERVAL1_SEC, uptime_sec) as f32);
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in tps task: {:?}", e));

    runtime.spawn(tps1_task);
}

pub fn spawn_tps2_tasks(runtime: &mut Runtime, args: &Arguments) {
    args.nodes.iter().for_each(|n| spawn_tps2_task(runtime, n));
}

pub fn spawn_tps2_task<'a>(runtime: &mut Runtime, node: &IctNode) {
    let interval = Duration::from_secs(MOVING_AVG_INTERVAL2_SEC);

    let mut uptime_sec: u64 = 0;
    let init = Instant::now();

    let arrivals2 = node.arrivals2.clone();
    let metrics = node.metrics.clone();

    let tps2_task = Interval::new_interval(Duration::from_millis(TPS_UPDATE_INTERVAL_MS))
        .for_each(move |instant| {
            let window_start = instant - interval;
            {
                let mut queue = arrivals2.lock().unwrap();
                while queue.len() > 0 && queue.front().unwrap() < &window_start {
                    queue.pop_front();
                }

                uptime_sec = (instant - init).as_secs();
                {
                    metrics.lock().unwrap().tps_avg2 =
                        queue.len() as f32 / (min(MOVING_AVG_INTERVAL2_SEC, uptime_sec) as f32);
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in tps-2 task: {:?}", e));

    runtime.spawn(tps2_task);
}

pub fn spawn_stdout_task(runtime: &mut Runtime, args: &Arguments) {
    let mut m: Vec<Arc<Mutex<Metrics>>> = vec![];
    args.nodes.iter().for_each(|n| m.push(n.metrics.clone()));

    let stdout_task = Interval::new_interval(Duration::from_millis(STDOUT_UPDATE_INTERVAL_MS))
        .for_each(move |_| {
            display::print_tps(&m);
            display::print_tps2(&m);
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

    let mut msg = zmq::Message::new();
    let poller_task = Interval::new_interval(Duration::from_millis(100))
        .for_each(move |_| {
            let mut poll_items = [responder.as_poll_item(zmq::POLLIN)];

            zmq::poll(&mut poll_items, 10).unwrap();

            if poll_items[0].is_readable() && responder.recv(&mut msg, 0).is_ok() {
                let msg = std::str::from_utf8(&msg).unwrap();
                match msg {
                    TPS_REQUEST => {
                        info!("Received tps request (1 min).");
                        {
                            responder
                                .send(
                                    &format!("tps:{:.2}", metrics_move.lock().unwrap().tps_avg1),
                                    0,
                                )
                                .unwrap();
                        }
                    }
                    TPS2_REQUEST => {
                        info!("Received tps-2 request (10 min).");
                        {
                            responder
                                .send(
                                    &format!("tps2:{:.2}", metrics_move.lock().unwrap().tps_avg2),
                                    0,
                                )
                                .unwrap();
                        }
                    }

                    GRAPH_REQUEST => {
                        info!("Received graph request");

                        // create a future, that will render the graph and store it as png
                        let render_task = plot::render_tps_graph()
                            .and_then(|_| {
                                responder.send("graph:ok", 0).unwrap();
                                Ok(())
                            })
                            .map_err(|e| {
                                panic!("Error while responding to tps graph request: {:?}", e)
                            });

                        // TODO: start render task
                    }
                    _ => {
                        warn!("Received unknown request.");
                    }
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in poller task: {:?}", e));

    runtime.spawn(poller_task);
}
