use std::{fs::File, io::Read, path::Path};

use league_loader::LeagueWadLoader;
use serde::{de::DeserializeOwned, Serialize};
use tokio::{fs::File as AsyncFile, io::AsyncWriteExt};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Custom error: {0}")]
    Custom(String),

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Binrw(#[from] binrw::Error),

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
    let path = format!("assets/{}", path);
    // println!("√ {}", path);
    ensure_dir_exists(&path)?;
    let file = AsyncFile::create(path).await?;
    Ok(file)
}

pub fn get_bin_path(path: &str) -> String {
    format!("{}.bin", path)
}

pub fn get_character_record_path(path: &str) -> String {
    let name = path.split("/").nth(1).unwrap();
    format!("Assets/Characters/{}/character_record", name)
}

pub fn get_struct_from_file<T: DeserializeOwned>(path: &str) -> Result<T, Error> {
    let mut file = File::open(format!("assets/{}", &get_bin_path(path)))?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    let data = bincode::deserialize(&data)?;
    Ok(data)
}

pub async fn save_wad_entry_to_file(loader: &LeagueWadLoader, path: &str) -> Result<(), Error> {
    let buffer = loader.get_wad_entry_buffer_by_path(path)?;
    let mut file = get_asset_writer(&path).await?;
    file.write_all(&buffer).await?;
    file.flush().await?;
    Ok(())
}
