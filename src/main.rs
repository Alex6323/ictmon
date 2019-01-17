#![allow(unreachable_code)]
use tokio::{prelude::*, runtime::Runtime};

use std::thread;
use std::time::Duration;

use clap::load_yaml;
use clap::{App, ArgMatches};

mod constants;
mod nodes;
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
            create_nodes_from_single(
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

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let args = Arguments::from_matches(matches);

    println!(
        "*** Welcome to '{}' (Ict Network Monitor) {}. ***",
        APP_NAME, APP_VERSION
    );

    let mut runtime = Runtime::new().unwrap();

    spawn_receiver_tasks(&mut runtime, &args);

    // wait for the receiver tasks to be initialized properly before continuing
    thread::sleep(Duration::from_millis(INITIAL_SLEEP_MS));

    spawn_tps_tasks(&mut runtime, &args);

    if args.run_stdout_task == true {
        println!("\n");
        spawn_stdout_task(&mut runtime, &args);
    }

    if args.run_responder_task == true {
        spawn_responder_task(&mut runtime, &args);
    }

    runtime.shutdown_on_idle().wait().unwrap();
}
