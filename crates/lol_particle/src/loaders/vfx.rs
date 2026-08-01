use bevy::asset::{AssetLoader, LoadContext};
use bevy::image::ImageLoaderSettings;
use bevy::reflect::TypePath;
use lol_base_render::error::Error;
use lol_base_render::particle::{ConfigVfx, VfxTexture};

/// skin{N}_vfx.ron 的自定义加载器。
///
/// 因为 ConfigVfx 现已改为纯 serde 结构（不再走反射），所以这里以纯 RON 反序列化；
/// 又因为其中的粒子贴图必须以线性色彩空间采样，所以对每个 VfxTexture 走
/// load_context 的 with_settings(is_srgb=false) 生成嵌套依赖 handle，从而无需 .meta 旁车。
#[derive(Default, TypePath)]
pub struct ConfigVfxLoader;

impl AssetLoader for ConfigVfxLoader {
    type Asset = ConfigVfx;

    type Settings = ();

    type Error = Error;

    async fn load(
        &self,
        reader: &mut dyn bevy::asset::io::Reader,
        _settings: &Self::Settings,
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut buf = Vec::new();
        reader.read_to_end(&mut buf).await?;

        let mut config: ConfigVfx =
            ron::de::from_bytes(&buf).map_err(|e| Error::Parse(e.to_string()))?;

        // 遍历所有系统/发射器，把 VfxTexture 的路径解析为线性加载的 Handle<Image>
        for system in config.systems.values_mut() {
            let emitter_lists = [
                system.complex_emitter_definition_data.as_mut(),
                system.simple_emitter_definition_data.as_mut(),
            ];
            for emitters in emitter_lists.into_iter().flatten() {
                for emitter in emitters.iter_mut() {
                    resolve_texture(&mut emitter.texture, load_context, true);
                    resolve_texture(&mut emitter.particle_color_texture, load_context, false);

                    if let Some(distortion) = emitter.distortion_definition.as_mut() {
                        resolve_texture(&mut distortion.normal_map_texture, load_context, false);
                    }

                    if let Some(overrides) = emitter.material_override_definitions.as_mut() {
                        for material_override in overrides.iter_mut() {
                            resolve_texture(
                                &mut material_override.base_texture,
                                load_context,
                                true,
                            );
                        }
                    }

                    if let Some(texture_mult) = emitter.texture_mult.as_mut() {
                        resolve_texture(&mut texture_mult.texture_mult, load_context, false);
                    }
                }
            }
        }

        Ok(config)
    }

    fn extensions(&self) -> &[&str] {
        &["ron"]
    }
}

/// 将单个 VfxTexture 的 path 解析为线性（is_srgb=false）加载的 Handle<Image>。
/// 因为 with_settings 覆盖了 ImageLoader 设置，所以无需为每张贴图生成 .meta 旁车。
fn resolve_texture(
    texture: &mut Option<VfxTexture>,
    load_context: &mut LoadContext<'_>,
    is_srgb: bool,
) {
    if let Some(texture) = texture.as_mut() {
        let path = texture.path.clone();
        texture.handle = load_context
            .load_builder()
            .with_settings(move |settings: &mut ImageLoaderSettings| settings.is_srgb = is_srgb)
            .load(path);
    }
}
