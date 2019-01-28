use std::{
    cmp::min,
    time::{Duration, Instant},
};

use tokio::{prelude::*, runtime::Runtime, timer::Interval};

use log::*;

use crate::constants::*;
use crate::display;
use crate::plotter;
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

        subscribers.push((subscriber, node.events.clone()));
    });

    let mut msg = zmq::Message::new();
    let sub_poller_task = Interval::new_interval(Duration::from_millis(SUB_POLLER_INTERVAL_MS))
        .for_each(move |_| {
            let mut poll_items = vec![];
            subscribers.iter().for_each(|(subscriber, _)| {
                poll_items.push(subscriber.as_poll_item(zmq::POLLIN));
            });

            zmq::poll(&mut poll_items, 10).unwrap();

            subscribers
                .iter()
                .enumerate()
                .for_each(|(i, (subscriber, events))| {
                    if poll_items[i].is_readable() && subscriber.recv(&mut msg, 0).is_ok() {
                        let instant = Instant::now();

                        let mut events = events.lock().unwrap();

                        events.timestamps1.push_back(instant);
                        events.timestamps2.push_back(instant);
                    }
                });
            Ok(())
        })
        .map_err(|e| panic!("Error in poller task: {:?}", e));

    runtime.spawn(sub_poller_task);
}

pub fn spawn_tps_tasks(runtime: &mut Runtime, args: &Arguments) {
    args.nodes.iter().for_each(|n| spawn_tps_task(runtime, n));
}

pub fn spawn_tps_task<'a>(runtime: &mut Runtime, node: &IctNode) {
    let avg_interval1 = Duration::from_secs(MOVING_AVG_INTERVAL1_SEC);
    let avg_interval2 = Duration::from_secs(MOVING_AVG_INTERVAL2_SEC);

    let mut uptime_sec: u64 = 0;
    let init = Instant::now();

    let events = node.events.clone();
    let metrics = node.metrics.clone();

    let tps_task = Interval::new_interval(Duration::from_millis(TPS_UPDATE_INTERVAL_MS))
        .for_each(move |instant| {
            let timeframe1_start = instant - avg_interval1;
            let timeframe2_start = instant - avg_interval2;
            {
                let mut events = events.lock().unwrap();

                while events.timestamps1.len() > 0
                    && events.timestamps1.front().unwrap() < &timeframe1_start
                {
                    events.timestamps1.pop_front();
                }

                while events.timestamps2.len() > 0
                    && events.timestamps2.front().unwrap() < &timeframe2_start
                {
                    events.timestamps2.pop_front();
                }

                // NOTE: 'uptime' doesn't have to be very accurate, since it only prevents very off
                // calculation results during the very first avg-interval after app launch
                // It's unfortunate I have to call the 'min' function for all eternity, if I want to
                // prevent extra lines.
                uptime_sec = (instant - init).as_secs();
                {
                    let metrics = &mut metrics.lock().unwrap();

                    if metrics.tps_avg1.len() == METRICS_HISTORY {
                        metrics.tps_avg1.pop_front();
                    }

                    if metrics.tps_avg2.len() == METRICS_HISTORY {
                        metrics.tps_avg2.pop_front();
                    }

                    metrics.tps_avg1.push_back(
                        events.timestamps1.len() as f64
                            / (min(MOVING_AVG_INTERVAL1_SEC, uptime_sec) as f64),
                    );

                    metrics.tps_avg2.push_back(
                        events.timestamps2.len() as f64
                            / (min(MOVING_AVG_INTERVAL2_SEC, uptime_sec) as f64),
                    );
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in tps task: {:?}", e));

    runtime.spawn(tps_task);
}

pub fn spawn_stdout_task(runtime: &mut Runtime, args: &Arguments) {
    let mut metrics = vec![];
    args.nodes
        .iter()
        .for_each(|node| metrics.push(node.metrics.clone()));

    let stdout_task = Interval::new_interval(Duration::from_millis(STDOUT_UPDATE_INTERVAL_MS))
        .for_each(move |_| {
            display::print_tps(&metrics);
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
    let rep_poller_task = Interval::new_interval(Duration::from_millis(100))
        .for_each(move |_| {
            let mut poll_items = [responder.as_poll_item(zmq::POLLIN)];

            zmq::poll(&mut poll_items, 10).unwrap();

            if poll_items[0].is_readable() && responder.recv(&mut msg, 0).is_ok() {
                let msg_str = std::str::from_utf8(&msg).unwrap();
                match msg_str {
                    TPS_REQUEST => {
                        info!("Received 'tps' request (1 min).");
                        {
                            let tps = match metrics_move.lock().unwrap().tps_avg1.back() {
                                Some(&tps) => tps,
                                None => 0.0,
                            };

                            responder
                                .send(&format!("tps{}{:.2}", RESPONSE_SEPARATOR, tps), 0)
                                .unwrap();
                        }
                    }
                    TPS10_REQUEST => {
                        info!("Received 'tps10' request (10 min).");
                        {
                            let tps = match metrics_move.lock().unwrap().tps_avg2.back() {
                                Some(&tps) => tps,
                                None => 0.0,
                            };

                            responder
                                .send(&format!("tps10{}{:.2}", RESPONSE_SEPARATOR, tps), 0)
                                .unwrap();
                        }
                    }

                    GRAPH_REQUEST => {
                        info!("Received graph request");
                        {
                            let metrics = metrics_move.lock().unwrap();

                            let data1 = metrics
                                .tps_avg1
                                .iter()
                                .enumerate()
                                .map(|(i, &d)| (i as f64, d))
                                .collect::<Vec<(f64, f64)>>();

                            let data2 = metrics
                                .tps_avg2
                                .iter()
                                .enumerate()
                                .map(|(i, &d)| (i as f64, d))
                                .collect::<Vec<(f64, f64)>>();

                            // NOTE: I cannot make the 'rendering' an asynchronous task, until I know
                            // a way to safely share the responder zmq socket
                            let result = match plotter::render_graph(&data1, &data2) {
                                Ok(filename) => filename,
                                Err(_) => String::from(""),
                            };

                            // For now we just notify the requester, that we rendered and stored the graph, because
                            // that's enough for what we need right now (local Discord bot). To make it more useful
                            // Ictmon should send the result base64 encoded to the requester.
                            {
                                responder
                                    .send(&format!("graph{}{}", RESPONSE_SEPARATOR, result), 0)
                                    .unwrap();
                            }
                        }
                    }
                    _ => {
                        warn!("Received unknown request. {}", msg_str);
                        {
                            responder.send(&format!("unknown:{}", msg_str), 0).unwrap();
                        }
                    }
                }
            }
            Ok(())
        })
        .map_err(|e| panic!("Error in poller task: {:?}", e));

    runtime.spawn(rep_poller_task);
}
