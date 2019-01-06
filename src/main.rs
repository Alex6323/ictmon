use std::env;
use std::process;
use std::thread;
use std::time::{Duration, Instant};

const ADDRESS: &str = "tcp://192.168.1.5"; //change this to localhost
const PORT: u16 = 5560;
const CHANNEL_TX: &'static str = "tx";

struct Arguments {
    program: String,
    address: String,
    port: u16,
}

impl Arguments {
    pub fn new(args: Vec<String>) -> Result<Self, &'static str> {
        if args.len() == 1 {
            Ok(Arguments {
                program: args[0].clone(),
                address: ADDRESS.to_string(),
                port: PORT,
            })
        } else if args.len() == 3 {
            Ok(Arguments {
                program: args[0].clone(),
                address: args[1].clone(),
                port: args[2].parse::<u16>().unwrap(),
            })
        } else {
            Err("Wrong number of arguments provided. Usage: ixi_zmq_listener <IP> <Port>")
        }
    }
}
fn main() {
    if let Ok(args) = Arguments::new(env::args().collect::<Vec<String>>()) {
        let context = zmq::Context::new();
        let subscriber = context.socket(zmq::SUB).unwrap();
        let address = format!("{}:{}", args.address, args.port);

        subscriber
            .connect(&address)
            .expect(&format!("Could not connect to publisher '{}'.", address));

        println!("Connected to address '{}'.", address);

        let subscription = CHANNEL_TX.to_string().into_bytes();
        subscriber.set_subscribe(&subscription).unwrap();

        println!("Subscribed to channel: '{:?}'", CHANNEL_TX);

        loop {
            let msg = subscriber.recv_msg(0).unwrap();
            let str_msg = std::str::from_utf8(&msg).unwrap();
            let parts: Vec<&str> = str_msg.split('|').collect();

            println!("hash: {}", parts[1]);
            //println!("signatureFragments: {}", parts[2]);
            //println!("address: {}", parts[3]);
            //println!("trytes: {}", parts[4]);
            //println!("isBundleHead: {}", parts[5]);
            //println!("timelockLowerBound: {}", parts[6]);
            //println!("timelockUpperBound: {}", parts[7]);
            //println!("attachmentTimestampLowerBound: {}", parts[8]);
            //println!("attachmentTimestamp: {}", parts[9]);
            //println!("attachmentTimestampUpperBound: {}", parts[10]);
            //println!("branchHash: {}", parts[11]);
            //println!("trunkHash:  {}", parts[12]);
            //println!("essence: {}", parts[13]);
            //println!("extraDataDigest: {}", parts[14]);
            //println!("nonce: {}", parts[15]);
            //println!("tag: {}", parts[16]);
            //println!("value: {}", parts[17]);
            //println!("decodedSignatureFragments: {}", parts[18]);
            println!("");
        }
    } else {
        process::exit(0);
    }
}
