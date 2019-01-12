use lazy_static::lazy_static;

use tokio::{prelude::*, runtime::Runtime};

use std::{
    collections::VecDeque,
    env, process,
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

const VERSION: &str = "v0.2.0-alpha";
const NAME: &str = "ictmon";
const PORT: u16 = 5560;
const INITIAL_SLEEP_MS: u64 = 1000;

lazy_static! {
    static ref ADDRESS: String = String::from("localhost");
}

mod tasks;
use crate::tasks::*;

pub struct Arguments {
    address: String,
    port: u16,
    run_stdout_task: bool,
    run_responder_task: bool,
}

impl Arguments {
    pub fn new(args: Vec<String>) -> Result<Self, String> {
        match args.len() {
            1 => Ok(Arguments {
                address: ADDRESS.clone(),
                port: PORT,
                run_stdout_task: true,
                run_responder_task: true,
            }),
            3 => Ok(Arguments {
                address: args[1].clone(),
                port: args[2].parse::<u16>().unwrap(),
                run_stdout_task: true,
                run_responder_task: true,
            }),
            _ => Err(format!(
                "Wrong number of arguments provided. Usage: ./{} <IP> <ZMQ-Port>",
                NAME
            )),
        }
    }
}

fn main() {
    let args: Arguments = match Arguments::new(env::args().collect::<Vec<String>>()) {
        Ok(a) => a,
        Err(s) => {
            println!("{}", s);
            process::exit(0);
        }
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
