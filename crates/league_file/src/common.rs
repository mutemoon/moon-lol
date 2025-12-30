use nom::bytes::complete::take;
use nom::number::complete::le_u32;
use nom::IResult;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizedStringU32 {
    pub len: u32,
    pub text: String,
}

impl SizedStringU32 {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, len) = le_u32(input)?;
        let (i, bytes) = take(len as usize)(i)?;
        let text = String::from_utf8_lossy(bytes)
            .trim_end_matches('\0')
            .to_string();
        Ok((i, SizedStringU32 { len, text }))
    }
}
