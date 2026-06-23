//! MoonLOL Web Server 库入口。
//!
//! 核心模块（domain/repository/cache/service/handlers）暴露为 lib。
//! handlers 层是 axum 路由的薄层：参数解析 → service → 序列化。

pub mod cache;
pub mod domain;
pub mod handlers;
pub mod repository;
pub mod service;
