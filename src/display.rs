use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::{
    cursor,
    style::{style, Color},
    terminal::{terminal, ClearType},
};

use crate::constants::{APP_NAME, APP_VERSION};
use crate::models::Metrics;
use crate::nodes::IctNode;

pub fn print_welcome() {
    cursor().hide().unwrap();
    terminal().clear(ClearType::All).unwrap();

    println!(
        "{}\n",
        style(format!(
            "Welcome to {} (Ict Node Monitor) {}. Connecting to node(s)...",
            APP_NAME, APP_VERSION
        ))
        .with(Color::Blue)
        .bold()
    );

    thread::sleep(Duration::from_millis(1000));
}

pub fn print_info(info: String) {
    println!("{}", style(format!("Info: {}", info)).with(Color::Yellow));
}

pub fn print_table(nodes: &Vec<IctNode>) {
    let (width, _) = terminal().terminal_size();
    let width = std::cmp::min(80_u16, width);
    let cursor = cursor();

    // TODO: print all borders here
    println!("+{}+", "-".repeat(width as usize - 2));
    for _ in 0..nodes.len() {
        println!("|{}|", " ".repeat(width as usize - 2));
    }
    println!("+{}+", "-".repeat(width as usize - 2));

    for i in 0..nodes.len() {
        cursor.goto(2, 3 + i as u16).unwrap();
        print!("{}", style(&nodes[i].name).with(Color::Yellow));
    }
}

pub fn print_tps(metrics: &Vec<Arc<Mutex<Metrics>>>) {
    let cursor = cursor();

    for i in 0..metrics.len() {
        cursor.goto(20, 3 + i as u16).unwrap();
        print!(
            "| {:.2} tps",
            style(metrics[i].lock().unwrap().tps_avg1).with(Color::Green)
        );
    }
}

pub fn print_shutdown() {
    let mut cursor = cursor();

    cursor.move_down(2);

    println!("\rTerminating...");

    thread::sleep(Duration::from_millis(1000));

    cursor.show().unwrap();
}
