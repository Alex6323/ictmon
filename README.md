# Ict Network Monitor (ictmon)

## About

'ictmon' is a small tool to monitor the activity of an Ict node running the [iota-ixi-zeromq](https://gitlab.com/Stefano_Core/iota-ixi-zeromq) extension module introduced by community member Stefano. The Ict node software itself is developed by the IOTA foundation and open-source. Its ongoing development can be followed here: [Ict](https://github.com/iotaledger/ict).

For now 'ictmon' simply calculates the current tps (transactions per second) running through the Ict node it listens to. I hope many more features like various types of filters will be added soon.

## Getting Started

### Compilation

* First you have to [install Rust](https://www.rust-lang.org/tools/install) for your system.
* Then clone the repository onto your local system and change into its directory.

    ```bash
    git clone https://github.com/Alex6323/ictmon.git && cd ictmon
    ```

* Finally compile the source using Rust's package manager.
    ```bash
    cargo build --release
    ```

### Usage

* From your local 'ictmon/' repository change into the 'release' directory.

    ```bash
    cd target/release
    ```

* If you are running your Ict node locally and use the default port 5560 you can simply type:
    ```bash
    ./ictmon
    ```

* Otherwise you have to provide 'ictmon' with an IP address and a port number:
    ```bash
    ./ictmon <IP> <ZMQ-PORT> 
    
    # Example:
    ./ictmon 192.168.1.9 5560
    ```

### Final Words

If you have any ideas or suggestions about features you'ld like to see in 'ictmon', please don't hesitate to contact me on the IOTA Discord server (/alex/#6323). 

Have fun!