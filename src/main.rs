#![allow(unreachable_code)]
//#![deny(warnings)]

use futures::{Future, Stream};
use tokio::runtime::Runtime;
use tokio_signal;

use std::{
    error::Error,
    thread,
    time::{Duration, Instant},
};

use clap::load_yaml;
use clap::{App, ArgMatches};

use log::*;

mod constants;
mod display;
mod models;
mod nodes;
mod plot;
mod tasks;

use crate::constants::*;
use crate::nodes::*;
use crate::tasks::*;

pub struct Arguments {
    nodes: Vec<IctNode>,
    topic: String,
    run_stdout_task: bool,
    run_responder_task: bool,
}

impl Arguments {
    pub fn from_matches(matches: ArgMatches) -> Self {
        let nodes = if matches.is_present(NODE_LIST_ARG) {
            create_nodes_from_file(ICT_LIST_FILE)
        } else {
            create_nodes_from_cli(
                matches.value_of(NAME_ARG).unwrap_or(DEFAULT_NAME),
                matches.value_of(ADDRESS_ARG).unwrap_or(DEFAULT_HOST),
                matches
                    .value_of(PORT_ARG)
                    .unwrap_or(DEFAULT_IXI_PORT)
                    .parse::<u16>()
                    .unwrap(),
            )
        };

        Arguments {
            nodes,
            topic: matches.value_of(TOPIC_ARG).unwrap_or(DEFAULT_TOPIC).into(),
            run_stdout_task: !matches.is_present(NO_STDOUT_ARG),
            run_responder_task: matches.is_present(API_ARG),
        }
    }
}

fn main() -> Result<(), Box<Error>> {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let args = Arguments::from_matches(matches);

    //let stop_signal = tokio_signal::ctrl_c().flatten_stream().take(1);
    //let receive_ctrl_c = stop_signal.for_each(|_| Ok(()));

    // TODO: create a new screen, that when app is exited closes as well
    display::print_welcome();
    display::print_table(&args.nodes);

    let mut runtime = Runtime::new().unwrap();

    info!("Connecting to nodes");
    let mut subscribers = vec![];

    args.nodes.iter().for_each(|node| {
        let addr = format!("tcp://{}:{}", node.address, node.port);

        let context = zmq::Context::new();
        let subscriber = context
            .socket(zmq::SUB)
            .expect("Error: Couldn't create zmq socket.");

        subscriber
            .connect(&addr)
            .expect("Error: Couldn't connect to node.");
        subscriber
            .set_subscribe(args.topic.as_bytes())
            .expect("Error: Couldn't subscribe to node.");

        subscribers.push(subscriber);
    });

    thread::sleep(Duration::from_millis(INITIAL_SLEEP_MS));

    info!("Starting tps tasks");
    spawn_tps_tasks(&mut runtime, &args);

    if args.run_stdout_task == true {
        info!("Starting stdout task");
        spawn_stdout_task(&mut runtime, &args);
    }

    if args.run_responder_task == true {
        info!("Starting responder task");
        spawn_responder_task(&mut runtime, &args);
    }

    let mut items = vec![];
    subscribers.iter().for_each(|subscriber| {
        items.push(subscriber.as_poll_item(zmq::POLLIN));
    });

    let mut msg = zmq::Message::new();
    loop {
        zmq::poll(&mut items, -1).unwrap();

        for i in 0..subscribers.len() {
            if items[i].is_readable() && subscribers[i].recv(&mut msg, 0).is_ok() {
                let mut queue = args.nodes[i].arrivals.lock().unwrap();
                queue.push_back(Instant::now());
            }
        }

        thread::sleep(Duration::from_millis(10));
    }

    //tokio::runtime::current_thread::block_on_all(receive_ctrl_c)?;

    //TODO: use tripwire
    runtime.shutdown_on_idle().wait().unwrap();
    //runtime.shutdown_now();

    //display::print_shutdown();

    Ok(())
}
