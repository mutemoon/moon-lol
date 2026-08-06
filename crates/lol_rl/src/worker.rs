use tokio::sync::Semaphore;

/// Controls concurrent training task execution.
/// CPU mode: up to `available_parallelism` tasks; CUDA mode: 1 task.
/// `MOON_LOL_MAX_CONCURRENT_TASKS` env var can override.
pub struct TrainingWorkerPool {
    semaphore: Semaphore,
}

impl TrainingWorkerPool {
    pub fn new(device_kind: crate::device::DeviceKind) -> Self {
        let max = max_concurrent_tasks(device_kind);
        Self {
            semaphore: Semaphore::new(max),
        }
    }

    pub fn max_concurrent(&self) -> usize {
        self.semaphore.available_permits()
    }

    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.semaphore
            .acquire()
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
