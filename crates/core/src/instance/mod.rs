mod config;
mod list;

pub use config::InstanceConfig;
pub use list::{InstanceSummary, discover_instances};

pub const INSTANCE_CONFIG_FILE: &str = "instance.cfg";
