mod compressed;
mod uncompressed;

use binrw::binread;
pub use compressed::*;
pub use uncompressed::*;

#[binread]
#[derive(Debug)]
#[br(little)]
pub enum AnimationFile {
    #[br(magic = b"r3d2anmd")]
    Uncompressed(UncompressedAnimationAsset),
    #[br(magic = b"r3d2canm")]
    Compressed(CompressedAnimationAsset),
}
