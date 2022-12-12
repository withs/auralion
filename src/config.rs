use log::debug;
use serde::{Deserialize, Serialize};
use std::{fs, path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub user_to_fetch: String,
    pub client_token: String,
    pub img_out_path: String,
    pub threads: usize,
}

pub fn load_config() -> Result<Config, String> {
    let cfg_file: &path::Path = path::Path::new("./config.toml");

    if !cfg_file.exists() {
        return Err("config.toml cannort be found at root".to_string());
    }

    let cfg_content: String = match fs::read_to_string(cfg_file) {
        Ok(ctn) => ctn,
        Err(err) => {
            debug!("{:?}", err);
            return Err("could not read file: config.toml".to_string());
        }
    };

    let config: Config = match toml::from_str(&cfg_content) {
        Ok(cfg) => cfg,
        Err(err) => {
            debug!("{:?}", err);
            return Err("failed to decode config file".to_string());
        }
    };

    debug!("{:?}", config);
    return Ok(config);
}
