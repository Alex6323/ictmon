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

use crate::constants::*;
use crate::models::{IctNode, Metrics};

pub fn print_welcome() {
    //cursor().hide().unwrap();
    terminal().clear(ClearType::All).unwrap();

    println!(
        "{}\n",
        style(format!(
            "Welcome to {} (Ict Node Monitor) {}.",
            APP_NAME, APP_VERSION
        ))
        .with(Color::Blue)
        .bold()
    );

    thread::sleep(Duration::from_millis(1000));
}

/*
pub fn print_info(info: String) {
    println!("{}", style(format!("Info: {}", info)).with(Color::Yellow));
}
*/

pub fn print_table(nodes: &[IctNode]) {
    let (width, _) = terminal().terminal_size();
    let width = std::cmp::min(MAX_TABLE_WIDTH, width);
    let cursor = cursor();

    // TODO: print header
    //println!("+{}+", "-".repeat(width as usize - 2));

    // TODO: print all borders here
    println!("+{}+", "-".repeat(width as usize - 2));
    for _ in 0..nodes.len() {
        println!("|{}|", " ".repeat(width as usize - 2));
    }
    println!("+{}+", "-".repeat(width as usize - 2));

    for (i, node) in nodes.iter().enumerate() {
        cursor
            .goto(TABLE_CONTENT_LEFT, TABLE_TPS_TOP + i as u16)
            .unwrap();
        print!("{}", style(&node.name).with(Color::Yellow));
    }

    reset_cursor(nodes.len() as u16 + TABLE_TPS_TOP + 1);
}

pub fn print_tps(metrics: &[Arc<Mutex<Metrics>>]) {
    let cursor = cursor();
    let (_, y) = cursor.pos();

    metrics.iter().enumerate().for_each(|(i, m)| {
        cursor
            .goto(
                TABLE_CONTENT_LEFT + TABLE_COLUMN_WIDTH,
                TABLE_TPS_TOP + i as u16,
            )
            .unwrap();

        let metrics = m.lock().unwrap();

        let avgs1 = &metrics.tps_avg1;
        let avgs2 = &metrics.tps_avg2;

        if let Some(avg) = avgs1.back() {
            print!("| {:.2} tps (1 min)", style(avg).with(Color::Green));
        }

        cursor
            .goto(
                TABLE_CONTENT_LEFT + 2 * TABLE_COLUMN_WIDTH,
                TABLE_TPS_TOP + i as u16,
            )
            .unwrap();

        if let Some(avg) = avgs2.back() {
            print!("| {:.2} tps (10 mins)", style(avg).with(Color::Green));
        }
    });

    stdout().flush().unwrap();

    reset_cursor(y);
}

fn reset_cursor(y: u16) {
    let cursor = cursor();
    cursor.goto(0, y).unwrap();
}

/*
pub fn print_shutdown() {
    let mut cursor = cursor();

    cursor.move_down(2);

    println!("\rTerminating...");

    thread::sleep(Duration::from_millis(1000));

    cursor.show().unwrap();
}
*/
