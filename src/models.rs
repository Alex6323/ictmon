use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

pub struct Metrics {
    pub tps_avg1: f32,
    pub tps_avg2: f32,
}

pub struct IctNode {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub arrivals: Arc<Mutex<VecDeque<Instant>>>,
    pub arrivals2: Arc<Mutex<VecDeque<Instant>>>,
    pub metrics: Arc<Mutex<Metrics>>,
}
