use std::sync::mpsc::{Receiver, Sender, channel};
use std::thread::{self, JoinHandle};

use crate::fiora_vs_riven::{FioraVsRivenAction, FioraVsRivenEnv, FioraVsRivenObs, StepResult};

enum EnvWorkerCmd {
    Reset,
    Step(FioraVsRivenAction),
    Stop,
}

enum EnvWorkerResp {
    Obs(FioraVsRivenObs),
    Step(StepResult),
}

pub struct EnvWorker {
    tx: Sender<EnvWorkerCmd>,
    rx: Receiver<EnvWorkerResp>,
    _handle: JoinHandle<()>,
}

impl EnvWorker {
    pub fn new(max_steps: usize) -> Self {
        let (tx_cmd, rx_cmd) = channel::<EnvWorkerCmd>();
        let (tx_resp, rx_resp) = channel::<EnvWorkerResp>();

        let handle = thread::spawn(move || {
            let mut env = FioraVsRivenEnv::new(max_steps);
            while let Ok(cmd) = rx_cmd.recv() {
                match cmd {
                    EnvWorkerCmd::Reset => {
                        let obs = env.reset();
                        if tx_resp.send(EnvWorkerResp::Obs(obs)).is_err() {
                            break;
                        }
                    }
                    EnvWorkerCmd::Step(act) => {
                        let res = env.step(act);
                        if tx_resp.send(EnvWorkerResp::Step(res)).is_err() {
                            break;
                        }
                    }
                    EnvWorkerCmd::Stop => break,
                }
            }
        });

        Self {
            tx: tx_cmd,
            rx: rx_resp,
            _handle: handle,
        }
    }

    pub fn reset(&self) -> FioraVsRivenObs {
        let _ = self.tx.send(EnvWorkerCmd::Reset);
        match self.rx.recv().expect("worker response failed") {
            EnvWorkerResp::Obs(obs) => obs,
            _ => panic!("unexpected worker response"),
        }
    }

    pub fn step(&self, action: FioraVsRivenAction) -> StepResult {
        let _ = self.tx.send(EnvWorkerCmd::Step(action));
        match self.rx.recv().expect("worker response failed") {
            EnvWorkerResp::Step(res) => res,
            _ => panic!("unexpected worker response"),
        }
    }
}

impl Drop for EnvWorker {
    fn drop(&mut self) {
        let _ = self.tx.send(EnvWorkerCmd::Stop);
    }
}

/// Runs N independent Bevy environment instances in parallel across OS threads.
/// Per docs/game/facts/bevy.md: each thread owns a single-threaded Bevy App instance,
/// avoiding Bevy multi-threading context switching while maximizing CPU throughput.
pub struct ParallelFioraVsRivenEnvs {
    workers: Vec<EnvWorker>,
}

impl ParallelFioraVsRivenEnvs {
    pub fn new(num_envs: usize, max_steps: usize) -> Self {
        let workers = (0..num_envs).map(|_| EnvWorker::new(max_steps)).collect();
        Self { workers }
    }

    pub fn reset_all(&self) -> Vec<FioraVsRivenObs> {
        self.workers.iter().map(|w| w.reset()).collect()
    }

    pub fn step_all(&self, actions: &[FioraVsRivenAction]) -> Vec<StepResult> {
        assert_eq!(actions.len(), self.workers.len());
        self.workers
            .iter()
            .zip(actions.iter())
            .map(|(w, &a)| w.step(a))
            .collect()
    }

    pub fn len(&self) -> usize {
        self.workers.len()
    }

    pub fn is_empty(&self) -> bool {
        self.workers.is_empty()
    }
}
