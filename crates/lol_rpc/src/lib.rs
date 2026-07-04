use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use lol_client::protocol::WsResponse;
use serde::de::DeserializeOwned;
use serde_json::Value;

#[derive(Event)]
pub struct CommandWsRequest<T> {
    pub id: u64,
    pub params: T,
    pub response: Arc<Mutex<Option<WsResponse>>>,
}

/// 运行期 RPC 注册表：各业务模块在自身 `Plugin::build` 中注册命令，
/// 避免所有参数类型集中定义在本 crate。
#[derive(Resource, Default)]
pub struct RpcRegistry {
    handlers: HashMap<
        &'static str,
        Arc<dyn Fn(&mut World, u64, Value, &Arc<Mutex<Option<WsResponse>>>) + Send + Sync>,
    >,
}

impl RpcRegistry {
    /// 注册一条命令：将命令名 `name` 映射到 `trigger::<T>`，
    /// `T` 在闭包内被擦除为动态分发。
    pub fn register<T: DeserializeOwned + Send + Sync + 'static>(&mut self, name: &'static str) {
        self.handlers.insert(
            name,
            Arc::new(|world, id, params, response| {
                trigger::<T>(world, id, params, response);
            }),
        );
    }
}

/// Bevy App 扩展：让各插件就地注册 RPC 命令。
pub trait RpcAppExt {
    /// 注册一条 RPC 命令（首次调用会初始化 `RpcRegistry` 资源）。
    fn register_rpc<T: DeserializeOwned + Send + Sync + 'static>(&mut self, name: &'static str);
}

impl RpcAppExt for App {
    fn register_rpc<T: DeserializeOwned + Send + Sync + 'static>(&mut self, name: &'static str) {
        self.init_resource::<RpcRegistry>();
        self.world_mut()
            .resource_mut::<RpcRegistry>()
            .register::<T>(name);
    }
}

pub fn trigger<T: DeserializeOwned + Send + Sync + 'static>(
    world: &mut World,
    id: u64,
    params: Value,
    response: &Arc<Mutex<Option<WsResponse>>>,
) {
    let params: T = match serde_json::from_value(params) {
        Ok(p) => p,
        Err(e) => {
            *response.lock().unwrap_or_else(|err| err.into_inner()) =
                Some(WsResponse::err(id, format!("无效参数: {}", e)));
            return;
        }
    };
    world.trigger(CommandWsRequest {
        id,
        params,
        response: response.clone(),
    });
}

pub fn dispatch(world: &mut World, id: u64, cmd: &str, params: Value) -> WsResponse {
    let response: Arc<Mutex<Option<WsResponse>>> = Arc::new(Mutex::new(None));
    let handler = world
        .get_resource::<RpcRegistry>()
        .and_then(|r| r.handlers.get(cmd).cloned());
    if let Some(h) = handler {
        h(world, id, params, &response);
    }

    let lock = response.lock().unwrap_or_else(|e| e.into_inner());
    lock.clone()
        .unwrap_or_else(|| WsResponse::err(id, format!("未知指令: {}", cmd)))
}

// ── Shared Helpers ──

pub fn respond<T>(event: &CommandWsRequest<T>, result: Result<Value, String>) {
    if let Ok(mut lock) = event.response.lock() {
        *lock = Some(match result {
            Ok(data) => WsResponse::ok_with_data(event.id, data),
            Err(e) => WsResponse::err(event.id, e),
        });
    }
}

pub fn resolve_target(
    entity_id: Option<u64>,
    exists_fn: impl FnOnce(Entity) -> bool,
    fallback_fn: impl FnOnce() -> Option<Entity>,
) -> Result<Entity, String> {
    if let Some(eid) = entity_id {
        let ent = Entity::from_bits(eid);
        if exists_fn(ent) {
            Ok(ent)
        } else {
            Err(format!("未找到指定的英雄实体 ID: {}", eid))
        }
    } else {
        fallback_fn().ok_or_else(|| "未找到存活的英雄实体".to_string())
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_target() {
        let exists = |e: Entity| e.to_bits() == 42;
        let fallback = || Some(Entity::from_bits(42));

        // Test with explicit existing entity
        assert_eq!(
            resolve_target(Some(42), exists, fallback),
            Ok(Entity::from_bits(42))
        );

        // Test with explicit non-existing entity
        assert!(resolve_target(Some(99), exists, fallback).is_err());

        // Test fallback
        assert_eq!(
            resolve_target(None, exists, fallback),
            Ok(Entity::from_bits(42))
        );

        // Test fallback failing
        assert!(resolve_target(None, exists, || None).is_err());
    }
}
