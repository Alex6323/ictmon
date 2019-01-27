# Ict Node Monitor (Ictmon)

## About

*Ictmon* is a small tool to monitor the activity of an Ict node running the [Ictmon.ixi](https://github.com/Alex6323/Ictmon.ixi) extension module, that was specifically designed to be able to serve interesting data to *Ictmon*. The open-source Ict node software itself is developed by the IOTA foundation. Its ongoing development can be followed here: [Ict](https://github.com/iotaledger/ict).

## Features
* inbound TPS (1 minute moving average)
* inbound TPS (10 minutes moving average)
* TPS graph
* ZeroMQ API (with `--api` argument)
* Muliple nodes monitoring (with `--node-list` argument)

For now *Ictmon* simply calculates the current inbound tps (transactions per second) of the Ict node it listens to. I hope many more features like various types of transaction filters, multi-node support and other interesting metrics will be added in the near future.

## Running the precompiled binary

In case you are running an Ubuntu based operating system, I provide a 64bit binary under the *releases* tab. The binary might work on other linux distributions as well (I haven't tested it yet). Before executing the binary, please compare one of the checksums to make sure it has not been altered. You probably will have to make it executable after downloading it:

```bash
$ sudo chmod +x ./ictmon
```

 When finished, skip the compilation section and head straight for the *Usage* section. However, I still recommend compiling it yourself. You are running the binary at your own risk.

## Compiling *ictmon* yourself (recommended)

### Prerequisites

* First you have to [install Rust](https://www.rust-lang.org/tools/install) for your specific platform. Please make sure, that your Rust installation is at least of version 1.31.0. You can check this by typing:
    ```bash
    $ rustc --version
    ```

* You will also need to have *libc*, *pkg-config*, *libzmq3-dev* and *git* installed:
    ```bash
    $ sudo apt install build-essential pkg-config libzmq3-dev git
    ```

* Then clone the repository to your local machine and change into its directory:

    ```bash
    $ git clone https://github.com/Alex6323/ictmon.git && cd ictmon/
    ```

### Compilation

* Compile the source using Rust's package manager *cargo*:

    ```bash
    $ cargo build --release
    ```

## Usage

* Change into the directory where *ictmon* is located. In case you compiled it yourself, you will find the created binary in *ictmon/target/release*

* If you are running your Ict node locally and use the default port 5560 you can simply type:
    ```bash
    $ ./ictmon
    ```

* Otherwise you have to provide *ictmon* with an IP address and a port number:
    ```bash
    $ ./ictmon <IP> <ZMQ-PORT> 
    
    # Example:
    $ ./ictmon 192.168.1.9 5560
    ```

## CLI

## WIP 

* CPU load + Mem usage of an Ict node
* Outbound transactions monitoring
* Interop with Report.ixi to display neighbor stats
* Webinterface
* Transaction visualizer
* Transaction arrival latency across multiple nodes

## Closing Words

If you have any ideas or suggestions about features you'ld like to see in *Ictmon*, please don't hesitate to contact me on the IOTA Discord server (/alex/#6323). 

Have fun!