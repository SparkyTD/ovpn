use std::path::Path;
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use tokio::fs;
use uuid::Uuid;
use crate::paths::CONFIGS_PATH;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ConfigEntry {
    pub name: String,
    pub guid: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigIndex {
    entries: Vec<ConfigEntry>,
}

impl ConfigIndex {
    pub async fn load(index_path: &str) -> Result<Self> {
        if Path::new(index_path).exists() {
            let data = fs::read_to_string(index_path).await?;
            let index: ConfigIndex = serde_json::from_str(&data)?;
            Ok(index)
        } else {
            Ok(ConfigIndex {
                entries: Vec::new(),
            })
        }
    }

    pub async fn save(&self, index_path: &str) -> Result<()> {
        let data = serde_json::to_string_pretty(&self)?;
        fs::write(index_path, data).await?;
        Ok(())
    }

    pub fn get_entries(&self) -> &Vec<ConfigEntry> {
        return &self.entries;
    }

    pub fn get_entries_mut(&mut self) -> &mut Vec<ConfigEntry> {
        return &mut self.entries;
    }
}

pub struct ConfigManager {
    index: ConfigIndex,
}

impl ConfigManager {
    pub async fn new() -> Result<ConfigManager> {
        let index_path = Self::get_index_path().await?;
        let index = ConfigIndex::load(index_path.as_str()).await?;

        return Ok(ConfigManager { index });
    }

    pub async fn import(&mut self, path: String, name: String) -> Result<Box<ConfigEntry>> {
        if let Some(_) = self.index.entries.iter().find(|e| e.name == name) {
            return Err(anyhow!("A configuration with the same name already exists."));
        }

        let guid = Uuid::new_v4().to_string();

        let entries = self.index.get_entries_mut();
        let entry = ConfigEntry {
            guid,
            name,
        };
        let entry_clone = entry.clone();
        let config_path = Self::get_config_path_and_check(&entry_clone).await?;
        fs::copy(path, config_path.as_ref()).await?;

        entries.push(entry.clone());

        let index_path = Self::get_index_path().await?;
        self.index.save(index_path.as_str()).await?;

        return Ok(Box::new(entry));
    }

    pub async fn delete(&mut self, config_name: String) -> Result<()> {
        match self.index.entries.iter().find(|e| e.name == config_name) {
            Some(entry) => {
                let config_path = Self::get_config_path_and_check(&entry).await?;
                fs::remove_file(config_path.as_ref()).await?;

                let entries = self.index.get_entries_mut();
                entries.retain(|e| e.name != config_name);

                let index_path = Self::get_index_path().await?;
                self.index.save(index_path.as_str()).await?;

                Ok(())
            },
            None => return Err(anyhow!("The specified configuration cannot be found."))
        }
    }

    pub async fn get_by_name(&self, config_name: String) -> Result<Box<ConfigEntry>> {
        return match self.index.entries.iter().find(|e| e.name == config_name) {
            Some(entry) => {
                let config_path = Self::get_config_path_and_check(&entry).await?;

                match Path::new(&config_path.as_ref()).exists() {
                    true => Ok(Box::new(entry.clone())),
                    false => return Err(anyhow!("The specified configuration cannot be found."))
                }
            },
            None => Err(anyhow!("The specified configuration cannot be found."))
        };
    }

    pub async fn get_config_path_and_check(entry: &ConfigEntry) -> Result<Box<String>> {
        let config_path = Self::get_config_path(entry);

        if !Path::new(&CONFIGS_PATH).exists() {
            fs::create_dir_all(&CONFIGS_PATH).await?;
        }

        return Ok(Box::new(config_path));
    }

    pub fn get_config_path(entry: &ConfigEntry) -> String {
        return format!("{}/{}.conf", CONFIGS_PATH, entry.guid);
    }

    pub async fn get_config_text(entry: &ConfigEntry) -> Result<String> {
        let config_path = Self::get_config_path(entry);
        match fs::read_to_string(config_path).await {
            Ok(text) => Ok(text),
            Err(_) => Err(anyhow!("Cannot read the configuration file."))
        }
    }

    async fn get_index_path() -> Result<String> {
        let index_path = format!("{}/{}", CONFIGS_PATH, "index.json");

        if !Path::new(&index_path).exists() {
            fs::create_dir_all(CONFIGS_PATH).await?;
        }

        return Ok(index_path);
    }

    pub fn get_index(&self) -> &ConfigIndex {
        return &self.index;
    }
}