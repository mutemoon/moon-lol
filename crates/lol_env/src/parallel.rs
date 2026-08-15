use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

use crate::traits::{RlEnvironment, StepResult};

enum WorkerCmd<A> {
    Reset,
    Step(A),
    Stop,
}

enum WorkerResp<O> {
    Obs(O),
    Step(StepResult<O>),
}

pub struct EnvWorker<E: RlEnvironment> {
    tx: Sender<WorkerCmd<E::Action>>,
    rx: Receiver<WorkerResp<E::Obs>>,
    _handle: JoinHandle<()>,
}

impl<E: RlEnvironment> EnvWorker<E> {
    pub fn new(max_steps: usize) -> Self {
        let (tx_cmd, rx_cmd) = channel::<WorkerCmd<E::Action>>();
        let (tx_resp, rx_resp) = channel::<WorkerResp<E::Obs>>();

        let handle = thread::spawn(move || {
            let mut env = E::new(max_steps);
            while let Ok(cmd) = rx_cmd.recv() {
                match cmd {
                    WorkerCmd::Reset => {
                        let obs = env.reset();
                        if tx_resp.send(WorkerResp::Obs(obs)).is_err() {
                            break;
                        }
                    }
                    WorkerCmd::Step(act) => {
                        let res = env.step(act);
                        if tx_resp.send(WorkerResp::Step(res)).is_err() {
                            break;
                        }
                    }
                    WorkerCmd::Stop => break,
                }
            }
        });

        Self {
            tx: tx_cmd,
            rx: rx_resp,
            _handle: handle,
        }
    }

    /// 发送命令但不等待响应（用于批量并行调度）。
    fn send_cmd(&self, cmd: WorkerCmd<E::Action>) {
        let _ = self.tx.send(cmd);
    }

    fn recv_obs(&self) -> E::Obs {
        match self.rx.recv().expect("worker response failed") {
            WorkerResp::Obs(obs) => obs,
            _ => panic!("unexpected worker response"),
        }
    }

    fn recv_step(&self) -> StepResult<E::Obs> {
        match self.rx.recv().expect("worker response failed") {
            WorkerResp::Step(res) => res,
            _ => panic!("unexpected worker response"),
        }
    }

    pub fn reset(&self) -> E::Obs {
        self.send_cmd(WorkerCmd::Reset);
        self.recv_obs()
    }

    pub fn step(&self, action: E::Action) -> StepResult<E::Obs> {
        self.send_cmd(WorkerCmd::Step(action));
        self.recv_step()
    }
}

impl<E: RlEnvironment> Drop for EnvWorker<E> {
    fn drop(&mut self) {
        let _ = self.tx.send(WorkerCmd::Stop);
    }
}

/// Runs N independent Bevy environment instances in parallel across OS threads.
/// Per docs/game/facts/bevy.md: each thread owns a single-threaded Bevy App instance,
/// avoiding Bevy multi-threading context switching while maximizing CPU throughput.
///
/// `step_all` / `reset_all` 采用「先全部派发命令、再统一收集响应」的两段式调度，
/// 确保 N 个 worker 真正并发执行，而非逐个阻塞等待（旧实现为串行）。
pub struct ParallelEnvs<E: RlEnvironment> {
    workers: Vec<EnvWorker<E>>,
}

impl<E: RlEnvironment> ParallelEnvs<E> {
    pub fn new(num_envs: usize, max_steps: usize) -> Self {
        let workers = (0..num_envs).map(|_| EnvWorker::new(max_steps)).collect();
        Self { workers }
    }

    /// 并行 reset 所有环境：先派发全部 Reset 命令，再统一收齐观测。
    pub fn reset_all(&self) -> Vec<E::Obs> {
        for w in &self.workers {
            w.send_cmd(WorkerCmd::Reset);
        }
        self.workers.iter().map(|w| w.recv_obs()).collect()
    }

    pub fn reset_one(&self, idx: usize) -> E::Obs {
        self.workers[idx].reset()
    }

    /// 并行 step 所有环境：先派发全部 Step 命令，再统一收齐结果。
    /// `actions.len()` 必须与 `len()` 一致。
    pub fn step_all(&self, actions: &[E::Action]) -> Vec<StepResult<E::Obs>> {
        for (i, &a) in actions.iter().enumerate() {
            self.workers[i].send_cmd(WorkerCmd::Step(a));
        }
        self.workers.iter().map(|w| w.recv_step()).collect()
    }

    pub fn len(&self) -> usize {
        self.workers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }
}

pub type ParallelFioraVsRivenEnvs = ParallelEnvs<crate::fiora_v0::FioraVsRivenEnv>;
pub type ParallelFioraVsRivenRealEnvs = ParallelEnvs<crate::fiora_v1::FioraVsRivenRealEnv>;
pub type ParallelFioraV2Envs = ParallelEnvs<crate::fiora_v2::FioraV2Env>;
