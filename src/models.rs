use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::Instant,
};

use crate::constants::*;

pub struct Metrics {
    pub tps_avg1: VecDeque<f64>,
    pub tps_avg2: VecDeque<f64>,
}

pub struct Events {
    pub timestamps1: VecDeque<Instant>,
    pub timestamps2: VecDeque<Instant>,
}

pub struct IctNode {
    pub name: String,
    pub address: String,
    pub port: u16,
    pub events: Arc<Mutex<Events>>,
    pub metrics: Arc<Mutex<Metrics>>,
}

impl Metrics {
    pub fn new() -> Self {
        Metrics {
            tps_avg1: VecDeque::with_capacity(METRICS_HISTORY),
            tps_avg2: VecDeque::with_capacity(METRICS_HISTORY),
        }
    }
}

impl Events {
    pub fn new() -> Self {
        Events {
            timestamps1: VecDeque::new(),
            timestamps2: VecDeque::new(),
        }
    }
}
