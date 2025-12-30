use nom::multi::count;
use nom::number::complete::{le_u32, le_u64};
use nom::{IResult, Parser};

use crate::common::SizedStringU32;

#[derive(Debug)]
pub struct LeagueShaderToc {
    pub magic: SizedStringU32,
    pub shader_count: u32,
    pub base_define_count: u32,
    pub bundled_shader_count: u32,
    pub shader_type: u32,
    pub base_defines_section_magic: SizedStringU32,
    pub base_defines: Vec<ShaderMacroDefinition>,
    pub shaders_section_magic: SizedStringU32,
    pub shader_hashes: Vec<u64>,
    pub shader_ids: Vec<u32>,
}

impl LeagueShaderToc {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, magic) = SizedStringU32::parse(input)?;
        let (i, shader_count) = le_u32(i)?;
        let (i, base_define_count) = le_u32(i)?;
        let (i, bundled_shader_count) = le_u32(i)?;
        let (i, shader_type) = le_u32(i)?;
        let (i, base_defines_section_magic) = SizedStringU32::parse(i)?;

        let (i, base_defines) =
            count(ShaderMacroDefinition::parse, base_define_count as usize).parse(i)?;
        let (i, shaders_section_magic) = SizedStringU32::parse(i)?;
        let (i, shader_hashes) = count(le_u64, shader_count as usize).parse(i)?;
        let (i, shader_ids) = count(le_u32, shader_count as usize).parse(i)?;

        Ok((
            i,
            LeagueShaderToc {
                magic,
                shader_count,
                base_define_count,
                bundled_shader_count,
                shader_type,
                base_defines_section_magic,
                base_defines,
                shaders_section_magic,
                shader_hashes,
                shader_ids,
            },
        ))
    }
}

#[derive(Debug)]
pub struct ShaderMacroDefinition {
    pub name: SizedStringU32,
    pub value: SizedStringU32,
}

impl ShaderMacroDefinition {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let (i, name) = SizedStringU32::parse(input)?;
        let (i, value) = SizedStringU32::parse(i)?;
        Ok((i, ShaderMacroDefinition { name, value }))
    }
}

#[derive(Debug, Clone)]
pub struct LeagueShaderChunk {
    pub files: Vec<SizedStringU32>,
}

impl LeagueShaderChunk {
    pub fn parse(input: &[u8]) -> IResult<&[u8], Self> {
        let mut files = Vec::new();
        let mut current_input = input;
        while !current_input.is_empty() {
            match SizedStringU32::parse(current_input) {
                Ok((i, s)) => {
                    files.push(s);
                    current_input = i;
                }
                Err(_) => break,
            }
        }
        Ok((current_input, LeagueShaderChunk { files }))
    }
}
