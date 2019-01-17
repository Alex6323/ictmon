use std::{
    collections::VecDeque,
    fs::File,
    io::{BufRead, BufReader},
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::Metrics;

pub struct IctNode {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub arrivals: Arc<Mutex<VecDeque<Instant>>>,
    pub metrics: Arc<Mutex<Metrics>>,
}

pub fn create_nodes_from_single(name: &str, address: &str, port: u16) -> Vec<IctNode> {
    let mut nodes = vec![];

    nodes.push(IctNode {
        name: name.into(),
        address: address.into(),
        port,
        arrivals: Arc::new(Mutex::new(VecDeque::new())),
        metrics: Arc::new(Mutex::new(Metrics(0.0))),
    });

    nodes
}

pub fn create_nodes_from_file(file: &str) -> Vec<IctNode> {
    let mut nodes: Vec<IctNode> = vec![];
    let buffered = BufReader::new(File::open(file).expect("File does not exist."));

    buffered
        .lines()
        .filter_map(|line| line.ok())
        .for_each(|line| {
            let parts = line.split(":").collect::<Vec<&str>>();
            nodes.push(IctNode {
                name: parts[0].into(),
                address: parts[1].into(),
                port: parts[2].parse::<u16>().unwrap(),
                arrivals: Arc::new(Mutex::new(VecDeque::new())),
                metrics: Arc::new(Mutex::new(Metrics(0.0))),
            })
        });

    nodes
}
