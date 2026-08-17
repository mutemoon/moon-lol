use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

use crate::traits::{RlEnvironment, StepResult};

enum WorkerCmd<A> {
    Reset,
    Step(Vec<A>),
    Stop,
}

enum WorkerResp<O> {
    Obs(Vec<O>),
    Step(Vec<StepResult<O>>),
}

pub struct EnvWorker<E: RlEnvironment> {
    tx: Sender<WorkerCmd<E::Action>>,
    rx: Receiver<WorkerResp<E::Obs>>,
    _handle: JoinHandle<()>,
}

impl<E: RlEnvironment> EnvWorker<E> {
    pub fn new() -> Self {
        Self::with_config(crate::traits::EnvConfig::default())
    }

    pub fn with_config(config: crate::traits::EnvConfig) -> Self {
        let (tx_cmd, rx_cmd) = channel::<WorkerCmd<E::Action>>();
        let (tx_resp, rx_resp) = channel::<WorkerResp<E::Obs>>();

        let handle = thread::spawn(move || {
            let mut env = E::with_config(config);
            while let Ok(cmd) = rx_cmd.recv() {
                match cmd {
                    WorkerCmd::Reset => {
                        let obs = env.reset();
                        if tx_resp.send(WorkerResp::Obs(obs)).is_err() {
                            break;
                        }
                    }
                    WorkerCmd::Step(acts) => {
                        let res = env.step(&acts);
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

    fn recv_obs(&self) -> Vec<E::Obs> {
        match self.rx.recv().expect("worker response failed") {
            WorkerResp::Obs(obs) => obs,
            _ => panic!("unexpected worker response"),
        }
    }

    fn recv_step(&self) -> Vec<StepResult<E::Obs>> {
        match self.rx.recv().expect("worker response failed") {
            WorkerResp::Step(res) => res,
            _ => panic!("unexpected worker response"),
        }
    }

    pub fn reset(&self) -> Vec<E::Obs> {
        self.send_cmd(WorkerCmd::Reset);
        self.recv_obs()
    }

    pub fn step(&self, actions: Vec<E::Action>) -> Vec<StepResult<E::Obs>> {
        self.send_cmd(WorkerCmd::Step(actions));
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
pub struct ParallelEnvs<E: RlEnvironment> {
    workers: Vec<EnvWorker<E>>,
}

impl<E: RlEnvironment> ParallelEnvs<E> {
    pub fn new(num_envs: usize) -> Self {
        let workers = (0..num_envs).map(|_| EnvWorker::new()).collect();
        Self { workers }
    }

    pub fn with_config(num_envs: usize, config: crate::traits::EnvConfig) -> Self {
        let workers = (0..num_envs)
            .map(|_| EnvWorker::with_config(config.clone()))
            .collect();
        Self { workers }
    }

    /// 并行 reset 所有环境：先派发全部 Reset 命令，再统一收齐多智能体观测。
    pub fn reset_all(&self) -> Vec<Vec<E::Obs>> {
        for w in &self.workers {
            w.send_cmd(WorkerCmd::Reset);
        }
        self.workers.iter().map(|w| w.recv_obs()).collect()
    }

    /// 展平并行 reset（将 N 个环境 × M 个 Agent 观测展平成长度为 N*M 的单列表）。
    pub fn reset_all_flat(&self) -> Vec<E::Obs> {
        self.reset_all().into_iter().flatten().collect()
    }

    pub fn reset_one(&self, idx: usize) -> Vec<E::Obs> {
        self.workers[idx].reset()
    }

    /// 并行 step 所有环境：输入每个环境的动作列表，返回每个环境的结果列表。
    pub fn step_all(&self, actions: &[Vec<E::Action>]) -> Vec<Vec<StepResult<E::Obs>>> {
        for (i, a) in actions.iter().enumerate() {
            self.workers[i].send_cmd(WorkerCmd::Step(a.clone()));
        }
        self.workers.iter().map(|w| w.recv_step()).collect()
    }

    /// 展平并行 step：输入 N*M 个动作，派发给各环境并展平返回 N*M 个 StepResult。
    pub fn step_all_flat(&self, flat_actions: &[E::Action]) -> Vec<StepResult<E::Obs>> {
        let num_agents = E::num_agents();
        let chunked: Vec<Vec<E::Action>> = flat_actions
            .chunks_exact(num_agents)
            .map(|c| c.to_vec())
            .collect();
        self.step_all(&chunked).into_iter().flatten().collect()
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
pub type ParallelFioraRivenSelfPlayEnvs =
    ParallelEnvs<crate::fiora_riven_selfplay::FioraRivenSelfPlayEnv>;
