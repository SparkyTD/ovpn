use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
    name: String,
    uuid: String
}

pub struct ConfigManager {
    config_entries: Vec<Config>
}

impl ConfigManager {
    pub fn new() -> ConfigManager {
        Self {
            config_entries: Vec::new(),
        }
    }
}