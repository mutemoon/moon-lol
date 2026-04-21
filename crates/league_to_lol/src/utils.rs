use std::fs::File;
use std::io::Read;
use std::path::Path;

use serde::de::DeserializeOwned;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Custom error: {0}")]
    Custom(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("{0}")]
    Bincode(#[from] bincode::Error),

    #[error("{0}")]
    LeagueLoader(#[from] league_loader::Error),
}

pub fn get_bin_path(path: &str) -> String {
    format!("{path}.bin")
}

pub fn get_struct_from_file<T: DeserializeOwned>(path: &str) -> Result<T, Error> {
    let mut file = File::open(format!("assets/{}", &get_bin_path(path)))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let data = bincode::deserialize(&data)?;
    Ok(data)
}
