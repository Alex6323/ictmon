use lazy_static::lazy_static;

use tokio::{prelude::*, runtime::Runtime};

use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

use clap::load_yaml;
use clap::{App, ArgMatches};

const VERSION: &str = "v0.2.0-alpha";
const NAME: &str = "ictmon";
const PORT: &str = "5560";
const INITIAL_SLEEP_MS: u64 = 1000;
const LOCALHOST: &str = "localhost";

mod tasks;
use crate::tasks::*;

pub struct Arguments {
    address: String,
    port: u16,
    run_stdout_task: bool,
    run_responder_task: bool,
}

impl Arguments {
    pub fn new(matches: ArgMatches) -> Self {
        Arguments {
            address: String::from(matches.value_of("address").unwrap_or(LOCALHOST)),
            port: matches
                .value_of("port")
                .unwrap_or(PORT)
                .parse::<u16>()
                .unwrap(),
            run_stdout_task: !matches.is_present("no-stdout"),
            run_responder_task: matches.is_present("api"),
        }
    }
}

fn main() {
    let yaml = load_yaml!("cli.yml");
    let matches = App::from_yaml(yaml).get_matches();

    let args = Arguments {
        address: String::from(matches.value_of("address").unwrap_or("localhost")),
        port: matches
            .value_of("port")
            .unwrap_or("5560")
            .parse::<u16>()
            .unwrap(),
        run_stdout_task: !matches.is_present("no-stdout"),
        run_responder_task: matches.is_present("api"),
    };

    println!(
        "*** Welcome to '{}' (Ict Network Monitor) {}. ***",
        NAME, VERSION
    );

    let arrivals: Arc<Mutex<VecDeque<Instant>>> = Arc::new(Mutex::new(VecDeque::new()));
    let metrics: Arc<Mutex<Metrics>> = Arc::new(Mutex::new(Metrics(0.0)));

    let mut runtime = Runtime::new().unwrap();

    spawn_receiver_task(&mut runtime, arrivals.clone(), &args);
    thread::sleep(Duration::from_millis(INITIAL_SLEEP_MS));
    spawn_tps_task(&mut runtime, arrivals.clone(), metrics.clone());

    if args.run_stdout_task == true {
        println!("\n");
        spawn_stdout_task(&mut runtime, metrics.clone());
    }

    if args.run_responder_task == true {
        spawn_responder_task(&mut runtime, metrics.clone());
    }

    runtime.shutdown_on_idle().wait().unwrap();
}
