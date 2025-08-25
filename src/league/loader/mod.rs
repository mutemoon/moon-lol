mod extract_class;
mod game;
mod map;
mod reader;
mod saver;
mod wad;
mod wad_parse;

pub use extract_class::*;
pub use game::*;
pub use map::*;
pub use reader::*;
pub use saver::*;
pub use wad::*;
pub use wad_parse::*;

use crate::league::BinDeserializerError;

#[derive(thiserror::Error, Debug)]
pub enum LeagueLoaderError {
    #[error("Could not load mesh: {0}")]
    Io(#[from] std::io::Error),

    #[error("Could not load texture: {0}")]
    BinRW(#[from] binrw::Error),

    #[error("Could not load texture: {0}")]
    Texture(#[from] bevy::image::TextureError),

    #[error("Could not serialize: {0}")]
    Bincode(#[from] bincode::Error),

    #[error("Could not deserialize: {0}")]
    BinDeserialize(#[from] BinDeserializerError),
}
