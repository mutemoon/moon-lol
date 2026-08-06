//! 全局 tokio runtime 桥接。
//!
//! client 的 UI 主线程与 gpui `cx.spawn` 的 AsyncApp executor 都不是 tokio
//! runtime。任何依赖 tokio IO 的服务（reqwest / sqlx / tokio::process / tokio::time /
//! tokio::task::spawn_blocking / WS）在 gpui executor 里直接调用都会 panic：
//! "there is no reactor running, must be called from the context of a Tokio 1.x runtime"。
//!
//! 这里提供一个全局 multi-thread tokio runtime，以及 `run_on_tokio` 桥接函数：
//! 把一段依赖 tokio 的异步闭包投递到全局 runtime 执行，用 oneshot 把结果桥回
//! 调用方的 async 上下文。服务层需要跑 tokio 代码的 async 方法统一走它。

use std::sync::OnceLock;

use tokio::runtime::Runtime;

/// 全局 tokio runtime（multi_thread，一次创建复用）。
pub fn tokio_runtime() -> &'static Runtime {
    static RT: OnceLock<Runtime> = OnceLock::new();
    RT.get_or_init(|| Runtime::new().expect("创建 tokio runtime 失败"))
}

/// 在全局 tokio runtime 内执行异步闭包，把结果桥回当前 async 上下文。
///
/// 适用于：调用方在 gpui AsyncApp（非 tokio）里，但闭包内部依赖 tokio 原语
/// （reqwest / sqlx / tokio::time / tokio::task::spawn_blocking 等）。
pub async fn run_on_tokio<T, F, Fut>(f: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Fut + Send + 'static,
    Fut: std::future::Future<Output = Result<T, String>> + Send + 'static,
{
    let (tx, rx) = tokio::sync::oneshot::channel();
    tokio_runtime().spawn(async move {
        let _ = tx.send(f().await);
    });
    rx.await.map_err(|e| format!("tokio 任务被取消: {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 模拟 UI 线程（非 tokio runtime 上下文）调用依赖 tokio IO 的闭包。
    /// 回归测试：修复前这里会 panic "there is no reactor running"。
    #[tokio::test(flavor = "multi_thread")]
    async fn run_on_tokio_executes_tokio_io_from_non_tokio_context() {
        let result = std::thread::spawn(|| {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                run_on_tokio(|| async {
                    tokio::time::sleep(std::time::Duration::from_millis(1)).await;
                    Ok::<_, String>(42)
                })
                .await
            })
        })
        .join()
        .unwrap();
        assert_eq!(result.unwrap(), 42);
    }

    /// 桥接错误：闭包返回 Err 时正确传播。
    #[tokio::test(flavor = "multi_thread")]
    async fn run_on_tokio_propagates_error() {
        let err = run_on_tokio(|| async { Err::<i32, _>("boom".to_string()) })
            .await
            .unwrap_err();
        assert_eq!(err, "boom");
    }
}
