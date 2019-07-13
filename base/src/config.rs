use serde::{Deserialize, Serialize};
use serde_millis;

use std::time::Duration;

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    #[serde(with = "serde_millis")]
    pub min_duration_hold: Duration,
    #[serde(with = "serde_millis")]
    pub min_duration_repeat: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            min_duration_hold: Duration::from_millis(350),
            min_duration_repeat: Duration::from_millis(100),
        }
    }
}