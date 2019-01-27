// App constants
pub const APP_VERSION: &str = "v0.2.0-alpha";
pub const APP_NAME: &str = "ictmon";

// Intervals
pub const INITIAL_SLEEP_MS: u64 = 1000;
pub const STDOUT_UPDATE_INTERVAL_MS: u64 = 1000;
pub const SUB_POLLER_INTERVAL_MS: u64 = 10;
pub const TPS_UPDATE_INTERVAL_MS: u64 = 1000;
pub const MOVING_AVG_INTERVAL1_SEC: u64 = 60;
pub const MOVING_AVG_INTERVAL2_SEC: u64 = 600;
pub const METRICS_HISTORY: usize = 3600;

// Default arguments
pub const DEFAULT_NAME: &str = "ict-0";
pub const DEFAULT_HOST: &str = "localhost";
pub const DEFAULT_IXI_PORT: &str = "5561";
pub const DEFAULT_API_PORT: &str = "5562";
pub const DEFAULT_TOPIC: &str = "in";

// CLI arguments
pub const NAME_ARG: &str = "name";
pub const ADDRESS_ARG: &str = "address";
pub const PORT_ARG: &str = "port";
pub const TOPIC_ARG: &str = "dir";
pub const NODE_LIST_ARG: &str = "file";
pub const API_ARG: &str = "api";
pub const NO_STDOUT_ARG: &str = "no-stdout";

// File names
pub const ICT_LIST_FILE: &str = "icts.txt";

// Requests
pub const TPS_REQUEST: &str = "tps";
pub const TPS2_REQUEST: &str = "tps2";
pub const GRAPH_REQUEST: &str = "graph";

// Display
pub const CURSOR_RESET_Y: u16 = 8;
