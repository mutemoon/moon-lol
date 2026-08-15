use std::sync::Arc;

use tokio::sync::Semaphore;

/// Controls concurrent training task execution.
/// CPU mode: up to `available_parallelism` tasks; CUDA mode: 1 task.
/// `MOON_LOL_MAX_CONCURRENT_TASKS` env var can override.
pub struct TrainingWorkerPool {
    semaphore: Arc<Semaphore>,
}

impl TrainingWorkerPool {
    pub fn new(device_kind: crate::device::DeviceKind) -> Self {
        let max = max_concurrent_tasks(device_kind);
        Self {
            semaphore: Arc::new(Semaphore::new(max)),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        self.semaphore.available_permits()
    }

    /// 获取一个可跨 `spawn_blocking` 移动的 owned permit，训练循环存活期间持有以限制并发。
    pub async fn acquire(&self) -> tokio::sync::OwnedSemaphorePermit {
        self.semaphore
            .clone()
            .acquire_owned()
            .await
            .expect("Semaphore 不应被关闭")
    }
}

fn max_concurrent_tasks(device_kind: crate::device::DeviceKind) -> usize {
    if let Ok(val) = std::env::var("MOON_LOL_MAX_CONCURRENT_TASKS") {
        if let Ok(n) = val.parse::<usize>() {
            return n.max(1);
        }
    }
    if device_kind == crate::device::DeviceKind::Cuda
        || (device_kind == crate::device::DeviceKind::Auto && cfg!(feature = "cuda"))
    {
        return 1;
    }
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}
