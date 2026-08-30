use std::collections::VecDeque;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use crate::engine::traits::QueueHealthStats;
use crate::engine::trajectory::WorkerTrajectory;

/// 异步训练有界环形轨迹缓冲队列（带策略版本标记、过期数据淘汰与健康度量）
pub struct TrajectoryRingBuffer<O> {
    capacity: usize,
    queue: Mutex<VecDeque<WorkerTrajectory<O>>>,
    not_empty: Condvar,
    is_closed: AtomicBool,
    total_pushed: AtomicUsize,
    total_dropped: AtomicUsize,
    recent_gap_sum: AtomicUsize,
    recent_gap_count: AtomicUsize,
}

impl<O> TrajectoryRingBuffer<O> {
    pub fn new(capacity: usize) -> Arc<Self> {
        Arc::new(Self {
            capacity: capacity.max(16),
            queue: Mutex::new(VecDeque::with_capacity(capacity)),
            not_empty: Condvar::new(),
            is_closed: AtomicBool::new(false),
            total_pushed: AtomicUsize::new(0),
            total_dropped: AtomicUsize::new(0),
            recent_gap_sum: AtomicUsize::new(0),
            recent_gap_count: AtomicUsize::new(0),
        })
    }

    /// 生产端（Sampler/Actor）：推入一条新生成的轨迹
    pub fn push(&self, traj: WorkerTrajectory<O>) -> bool {
        if self.is_closed.load(Ordering::Relaxed) {
            return false;
        }

        self.total_pushed.fetch_add(1, Ordering::Relaxed);
        let mut q = self.queue.lock().unwrap();

        // 环形缓冲：若队列已满，淘汰最旧的过期数据，保证新数据进入
        if q.len() >= self.capacity {
            let _ = q.pop_front();
            self.total_dropped.fetch_add(1, Ordering::Relaxed);
        }

        q.push_back(traj);
        self.not_empty.notify_one();
        true
    }

    /// 消费端（Learner）：拉取满批轨迹，并进行策略版本差检查与过期淘汰
    pub fn recv_rollout_batch(
        &self,
        target_steps: usize,
        current_version: usize,
        max_staleness: usize,
        timeout: Duration,
    ) -> anyhow::Result<(Vec<WorkerTrajectory<O>>, usize, QueueHealthStats)> {
        let start = Instant::now();
        let mut collected = Vec::new();
        let mut collected_samples = 0usize;
        let mut round_gap_sum = 0usize;
        let mut round_gap_count = 0usize;

        let mut q = self.queue.lock().unwrap();

        while collected_samples < target_steps {
            if self.is_closed.load(Ordering::Relaxed) && q.is_empty() {
                break;
            }

            while q.is_empty() {
                if self.is_closed.load(Ordering::Relaxed) {
                    break;
                }
                let elapsed = start.elapsed();
                if elapsed >= timeout {
                    break;
                }
                let remaining = timeout - elapsed;
                let (new_q, timeout_res) = self.not_empty.wait_timeout(q, remaining).unwrap();
                q = new_q;
                if timeout_res.timed_out() {
                    break;
                }
            }

            if q.is_empty() {
                break;
            }

            if let Some(traj) = q.pop_front() {
                let gap = current_version.saturating_sub(traj.policy_version);

                // 版本淘汰：若策略版本落后超过 max_staleness（纯 PPO 建议 <= 2~3），丢弃以防策略失效
                if gap > max_staleness && max_staleness > 0 {
                    self.total_dropped.fetch_add(1, Ordering::Relaxed);
                    continue;
                }

                round_gap_sum += gap;
                round_gap_count += 1;
                self.recent_gap_sum.fetch_add(gap, Ordering::Relaxed);
                self.recent_gap_count.fetch_add(1, Ordering::Relaxed);

                for b in &traj.buffers {
                    collected_samples += b.len();
                }
                collected.push(traj);
            }
        }

        let current_len = q.len();
        drop(q);

        let total_pushed = self.total_pushed.load(Ordering::Relaxed);
        let total_dropped = self.total_dropped.load(Ordering::Relaxed);
        let drop_ratio = if total_pushed > 0 {
            (total_dropped as f64 / total_pushed as f64) * 100.0
        } else {
            0.0
        };

        let avg_policy_gap = if round_gap_count > 0 {
            round_gap_sum as f64 / round_gap_count as f64
        } else {
            0.0
        };

        let stats = QueueHealthStats {
            drop_ratio,
            avg_policy_gap,
            queue_len: current_len,
            queue_capacity: self.capacity,
            total_pushed,
            total_dropped,
        };

        Ok((collected, collected_samples, stats))
    }

    /// 获取当前的健康度量快照
    pub fn health_stats(&self) -> QueueHealthStats {
        let q = self.queue.lock().unwrap();
        let queue_len = q.len();
        drop(q);

        let total_pushed = self.total_pushed.load(Ordering::Relaxed);
        let total_dropped = self.total_dropped.load(Ordering::Relaxed);
        let drop_ratio = if total_pushed > 0 {
            (total_dropped as f64 / total_pushed as f64) * 100.0
        } else {
            0.0
        };

        let gap_sum = self.recent_gap_sum.load(Ordering::Relaxed);
        let gap_cnt = self.recent_gap_count.load(Ordering::Relaxed);
        let avg_policy_gap = if gap_cnt > 0 {
            gap_sum as f64 / gap_cnt as f64
        } else {
            0.0
        };

        QueueHealthStats {
            drop_ratio,
            avg_policy_gap,
            queue_len,
            queue_capacity: self.capacity,
            total_pushed,
            total_dropped,
        }
    }

    pub fn close(&self) {
        self.is_closed.store(true, Ordering::Relaxed);
        self.not_empty.notify_all();
    }
}
