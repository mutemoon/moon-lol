//! Cache 层入口：声明所有 cache trait。
//! 每个 cache trait 提供 get/put/invalidate 语义，impl 有 Moka（生产）和 Noop（测试默认）。

pub mod model_provider_cache;

pub use model_provider_cache::{
    ModelProviderCache, MokaModelProviderCache, NoopModelProviderCache,
};
