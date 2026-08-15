use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::thread::{self, ThreadId};

use bevy::app::{FixedUpdate, PostUpdate, PreUpdate, Update};
use bevy::ecs::system::Res;
use bevy::prelude::Resource;
use lol_env::FioraVsRivenRealEnv;
use lol_env::fiora_v1::FioraVsRivenRealAction;

#[derive(Resource, Clone, Default)]
struct ThreadProbe {
    recorded_threads: Arc<Mutex<HashSet<ThreadId>>>,
}

fn probe_system(probe: Res<ThreadProbe>) {
    let current_tid = thread::current().id();
    probe.recorded_threads.lock().unwrap().insert(current_tid);
}

#[test]
fn test_env_runs_strictly_on_single_thread() {
    let caller_thread_id = thread::current().id();
    println!("📍 [Caller Thread] 外部调用线程 ID: {:?}", caller_thread_id);

    // 1. 初始化无头环境（内部已默认配置 SingleThreadedExecutor）
    let mut env = FioraVsRivenRealEnv::new(100);
    let _ = env.reset();

    // 2. 注入探针系统到 Bevy 的多个关键调度阶段
    let probe = ThreadProbe::default();
    let thread_records = probe.recorded_threads.clone();

    env.app.insert_resource(probe);
    env.app.add_systems(PreUpdate, probe_system);
    env.app.add_systems(FixedUpdate, probe_system);
    env.app.add_systems(Update, probe_system);
    env.app.add_systems(PostUpdate, probe_system);

    // 3. 执行多步 step
    let action = FioraVsRivenRealAction::from_encoding(&[0.0, 0.0, 0.0]);
    for _ in 1..=50 {
        let res = env.step(action.clone());
        if res.terminated || res.truncated {
            let _ = env.reset();
        }
    }

    // 4. 验证记录到的所有线程 ID
    let seen_threads = thread_records.lock().unwrap().clone();
    println!(
        "🔍 [Bevy Systems Thread Records] 内部系统记录到的所有线程集合: {:?}",
        seen_threads
    );

    assert_eq!(
        seen_threads.len(),
        1,
        "❌ 验证失败：内部 Systems 跑在了多个线程中！{:?}",
        seen_threads
    );

    assert!(
        seen_threads.contains(&caller_thread_id),
        "❌ 验证失败：内部 Systems 运行的线程不是外部调用者的线程！"
    );

    println!("✅ [验证通过] 该环境实例内的所有 Systems 100% 严格只在外部调用的单一线程上执行！");
}

#[test]
fn test_multiple_envs_isolated_on_their_own_threads() {
    let num_workers = 4;
    let mut handles = Vec::new();

    for worker_id in 0..num_workers {
        let handle = thread::spawn(move || {
            let my_tid = thread::current().id();
            let mut env = FioraVsRivenRealEnv::new(100);
            let _ = env.reset();

            let probe = ThreadProbe::default();
            let thread_records = probe.recorded_threads.clone();

            env.app.insert_resource(probe);
            env.app.add_systems(Update, probe_system);
            env.app.add_systems(FixedUpdate, probe_system);

            let action = FioraVsRivenRealAction::from_encoding(&[0.0, 0.0, 0.0]);
            for _ in 0..30 {
                let _ = env.step(action.clone());
            }

            let seen = thread_records.lock().unwrap().clone();
            assert_eq!(seen.len(), 1);
            assert!(seen.contains(&my_tid));

            (worker_id, my_tid, seen)
        });
        handles.push(handle);
    }

    let mut all_worker_threads = HashSet::new();
    for h in handles {
        let (worker_id, my_tid, seen) = h.join().unwrap();
        println!(
            "Worker #{}: 线程 ID {:?}，内部 Systems 运行线程: {:?}",
            worker_id, my_tid, seen
        );
        all_worker_threads.insert(my_tid);
    }

    assert_eq!(
        all_worker_threads.len(),
        num_workers,
        "4 个 Worker 应当各自运行在 4 个互相独立的 OS 线程上"
    );
    println!("✅ [验证通过] 多个环境实例在各自独立的线程上互不干扰、严格单线程闭环！");
}
