use std::{
    io::{stdout, Write},
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use crossterm::{
    cursor,
    style::{style, Color},
    terminal::{terminal, ClearType},
};

use crate::constants::{APP_NAME, APP_VERSION, CURSOR_RESET_Y};
use crate::models::{IctNode, Metrics};

pub fn print_welcome() {
    //cursor().hide().unwrap();
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

    reset_cursor();
}

pub fn print_tps(metrics: &Vec<Arc<Mutex<Metrics>>>) {
    let cursor = cursor();

    for i in 0..metrics.len() {
        cursor.goto(22, 3 + i as u16).unwrap();
        print!(
            "| {:.2} tps (1min)",
            style(metrics[i].lock().unwrap().tps_avg1).with(Color::Green)
        );
    }
    stdout().flush().unwrap();

    reset_cursor();
}

pub fn print_tps2(metrics: &Vec<Arc<Mutex<Metrics>>>) {
    let cursor = cursor();

    for i in 0..metrics.len() {
        cursor.goto(42, 3 + i as u16).unwrap();
        print!(
            "| {:.2} tps (10min)",
            style(metrics[i].lock().unwrap().tps_avg2).with(Color::Green)
        );
    }
    stdout().flush().unwrap();

    reset_cursor();
}

// TODO: make this dynamic
fn reset_cursor() {
    let cursor = cursor();
    cursor.goto(0, CURSOR_RESET_Y).unwrap();
}

pub fn print_shutdown() {
    let mut cursor = cursor();

    cursor.move_down(2);

    println!("\rTerminating...");

    thread::sleep(Duration::from_millis(1000));

    cursor.show().unwrap();
}
