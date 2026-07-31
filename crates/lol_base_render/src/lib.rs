//! 渲染侧共享基础设施：被 lol_render 与 lol_particle 共同依赖的内容下沉于此，
//! 因为粒子系统独立成 crate 后两者都需要 shader 布局 / 相机 / 粒子配置等基础类型，
//! 所以提取到本 crate 以避免循环引用。

pub mod animation;
pub mod camera;
pub mod error;
pub mod mesh;
pub mod mesh_shadow;
pub mod particle;
pub mod shader;
pub mod shader_layout;
pub mod ui;
pub mod ui_components;
