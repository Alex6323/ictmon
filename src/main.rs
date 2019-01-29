// NOTE: swap the inclusion of the following two lines before releases
//#![allow(dead_code, unused_variables, unused_imports, unused_mut)]
//#![deny(warnings)]

use futures::{Future, Stream};
use stream_cancel::Tripwire;
use tokio::runtime::Runtime;

use std::{env, error::Error, thread, time::Duration};

use clap::load_yaml;
use clap::{App, ArgMatches};

use log::*;
use pretty_env_logger;

mod constants;
mod display;
mod models;
mod nodes;
mod plotter;
mod tasks;

use crate::constants::*;
use crate::models::IctNode;
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
    env::set_var(APP_ENV_VAR, DEBUG_LEVEL);
    pretty_env_logger::init_custom_env(APP_ENV_VAR);

    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();
    let args = Arguments::from_matches(matches);

    // TODO: create a new screen, that when app is exited closes as well
    if args.run_stdout_task {
        display::print_welcome();
        display::print_table(&args.nodes);
    }

    let mut runtime = Runtime::new().unwrap();
    let (trigger, tripwire) = Tripwire::new();
    let shutdown_signal = tokio_signal::ctrl_c()
        .flatten_stream()
        .take(1)
        .for_each(|_| Ok(()));

    debug!("Starting poller task.");
    spawn_poller_task(&mut runtime, &args, tripwire.clone());

    thread::sleep(Duration::from_millis(INITIAL_SLEEP_MS));

    debug!("Starting tps tasks.");
    spawn_tps_tasks(&mut runtime, &args, tripwire.clone());

    if args.run_stdout_task {
        debug!("Starting stdout task.");
        spawn_stdout_task(&mut runtime, &args, tripwire.clone());
    }

    if args.run_responder_task {
        debug!("Starting responder task.");
        spawn_responder_task(&mut runtime, &args, tripwire.clone());
    }

    tokio::runtime::current_thread::block_on_all(shutdown_signal).unwrap();
    info!("Shutting down...");

    drop(trigger);
    runtime.shutdown_on_idle().wait().unwrap();

    thread::sleep(Duration::from_millis(1000));

    Ok(())
}
