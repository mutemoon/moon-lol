use std::collections::HashMap;

use bevy::asset::{AssetLoader, AssetServer, Handle, LoadContext};
use bevy::log::debug;
use bevy::reflect::TypePath;
use bevy::render::render_resource::ShaderStage;
use bevy::shader::{ShaderImport, Source, ValidateShader};
use league_file::shader::{LeagueShaderChunk, LeagueShaderToc};
use league_to_lol::shader::{convert_frag, convert_vert};
use league_utils::{get_shader_uuid_by_hash, hash_wad};
use lol_base::shader::ResourceShaderPackage;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::error::Error;

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct ShaderTocSettings(pub String);

#[derive(Default, TypePath)]
pub struct LeagueLoaderShaderToc;

impl AssetLoader for LeagueLoaderShaderToc {
    type Asset = ResourceShaderPackage;

    type Settings = ShaderTocSettings;

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let (_, shader_toc) =
            LeagueShaderToc::parse(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        let path = &settings.0;

        let mut handles = HashMap::new();

        let mut chunks = Vec::new();

        let mut max_struct = "".to_string();

        let mut max_uniform_sampler = "".to_string();

        for i in 0..((shader_toc.bundled_shader_count as f32 / 100.0).ceil() as usize) {
            let chunk_hash = hash_wad(&format!("{}_{}", path, i * 100));

            let chunk = load_context
                .read_asset_bytes(&format!("data/{:x}.lol", chunk_hash))
                .await
                .unwrap();

            let (_, shader_chunk) =
                LeagueShaderChunk::parse(&chunk).map_err(|e| Error::Parse(e.to_string()))?;

            for shader_file in shader_chunk.files.iter() {
                let content = shader_file.text.clone();

                // content.matches("struct")

                let re = Regex::new(r"struct[\w\W]*?\}").unwrap(); // 匹配 YYYY-MM-DD 日期格式
                let matches = re.find_iter(&content);

                for mat in matches {
                    if mat.as_str().len() > max_struct.len() {
                        max_struct = mat.as_str().to_string();
                    }
                }

                let re = Regex::new(r"uniform sampler2D[\w\W]*?\n\n").unwrap();
                let matches = re.find_iter(&content);

                for mat in matches {
                    if mat.as_str().len() > max_uniform_sampler.len() {
                        max_uniform_sampler = mat.as_str().to_string();
                    }
                }

                chunks.push(content);
            }
        }

        let mut shader_handles = Vec::new();
        for i in 0..shader_toc.bundled_shader_count {
            let mut content = chunks[i as usize].clone();

            let re = Regex::new(r"struct[\w\W]*?\}").unwrap(); // 匹配 YYYY-MM-DD 日期格式
            let matches = re.find_iter(&content);

            let ranges = matches.map(|mat| mat.range()).collect::<Vec<_>>();

            for range in ranges {
                content.replace_range(range, &max_struct);
            }

            let re = Regex::new(r"uniform sampler2D[\w\W]*?\n\n").unwrap();
            let matches = re.find_iter(&content);

            let ranges = matches.map(|mat| mat.range()).collect::<Vec<_>>();

            for range in ranges {
                content.replace_range(range, &max_uniform_sampler);
            }

            let converted = if shader_toc.shader_type == 0 {
                convert_vert(&content)
            } else {
                convert_frag(&content)
            };

            let source = if shader_toc.shader_type == 0 {
                Source::Glsl(converted.clone().into(), ShaderStage::Vertex)
            } else {
                Source::Glsl(converted.clone().into(), ShaderStage::Fragment)
            };

            use bevy::shader::Shader;
            let shader = Shader {
                path: path.clone(),
                imports: Default::default(),
                import_path: ShaderImport::Custom(path.clone()),
                source,
                additional_imports: Default::default(),
                shader_defs: Default::default(),
                file_dependencies: Default::default(),
                validate_shader: ValidateShader::Disabled,
            };

            shader_handles.push((
                converted.clone(),
                load_context.add_labeled_asset(i.to_string(), shader),
            ));
        }

        for (shader_index, shader_hash) in shader_toc.shader_hashes.into_iter().enumerate() {
            let shader_id = shader_toc.shader_ids[shader_index];

            let (converted, handle) = &shader_handles[shader_id as usize];

            if get_shader_uuid_by_hash(&path, shader_hash)
                == bevy::asset::uuid::Uuid::from_u128(0xdee3e40ffaa02909)
            {
                debug!("shader_id: {}", shader_id);
                debug!("converted: {}", converted);
            }

            handles.insert(shader_hash, handle.clone());
        }

        Ok(ResourceShaderPackage { handles })
    }

    fn extensions(&self) -> &[&str] {
        &["glsl"]
    }
}

pub trait AssetServerLoadShaderToc {
    fn load_shader_toc<'a>(&self, path: &str) -> Handle<ResourceShaderPackage>;
}

impl AssetServerLoadShaderToc for AssetServer {
    fn load_shader_toc<'a>(&self, path: &str) -> Handle<ResourceShaderPackage> {
        let original_path = path.to_string();
        self.load_with_settings(
            format!("data/{:x}.lol", hash_wad(path)),
            move |settings: &mut ShaderTocSettings| settings.0 = original_path.clone(),
        )
    }
}
