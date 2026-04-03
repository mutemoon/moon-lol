use std::fs::File;
use std::io::Read;
use std::path::Path;

use league_loader::LeagueWadLoaderTrait;
use serde::de::DeserializeOwned;
use serde::Serialize;
use thiserror::Error;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;

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

fn ensure_dir_exists(path: &str) -> Result<(), Error> {
    let dir = Path::new(path).parent().unwrap();
    if !dir.exists() {
        std::fs::create_dir_all(dir)?;
    }
    Ok(())
}

pub async fn save_struct_to_file<T: Serialize>(path: &str, data: &T) -> Result<(), Error> {
    let serialized = bincode::serialize(data)?;
    let mut file = get_asset_writer(path).await?;
    file.write_all(&serialized).await?;
    file.flush().await?;
    Ok(())
}

pub async fn get_asset_writer(path: &str) -> Result<AsyncFile, Error> {
    let path = format!("assets/{path}");
    // println!("√ {}", path);
    ensure_dir_exists(&path)?;
    let file = AsyncFile::create(path).await?;
    Ok(file)
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

pub async fn save_wad_entry_to_file(
    loader: &impl LeagueWadLoaderTrait,
    path: &str,
) -> Result<(), Error> {
    let buffer = loader.get_wad_entry_buffer_by_path(path)?;
    let mut file = get_asset_writer(&path).await?;
    file.write_all(&buffer).await?;
    file.flush().await?;
    Ok(())
}
