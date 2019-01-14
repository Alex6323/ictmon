use std::fs::File;
use std::io::{BufRead, BufReader};

const NODE_LIST_FILE: &str = "node_list.txt";

/* <name>:<ip-address>:<zmq-port>
 *
 */

#[derive(Debug)]
pub struct Node {
    name: String,
    ip_address: String,
    port: u16,
}

#[derive(Debug)]
pub struct Nodes {
    nodes: Vec<Node>,
}

impl Nodes {
    pub fn new_from_file(file: &str) -> Self {
        let mut nodes: Vec<Node> = vec![];
        let buffered = BufReader::new(File::open(file).expect("File does not exist."));

        buffered
            .lines()
            .filter_map(|line| line.ok())
            .for_each(|line| {
                let parts = line.split(":").collect::<Vec<&str>>();
                nodes.push(Node {
                    name: String::from(parts[0]),
                    ip_address: String::from(parts[1]),
                    port: parts[2].parse::<u16>().unwrap(),
                })
            });

        Nodes { nodes }
    }
}
