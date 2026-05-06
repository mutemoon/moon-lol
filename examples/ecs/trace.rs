use std::fs::File;
use std::io::Write;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use bevy::diagnostic::FrameCount;
use bevy::prelude::*;
use bevy::time::TimeUpdateStrategy;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
enum EventType {
    System,
    Observer,
    Frame,
    FixedUpdateLoop,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
struct ChromeTraceEvent {
    name: String,
    cat: String,
    ph: String, // Phase: "X" for Complete Event
    ts: f64,    // Microseconds
    dur: f64,   // Microseconds
    pid: u32,
    tid: String,
    args: serde_json::Value,
}

#[derive(Resource, Default)]
struct TraceResource {
    events: Arc<Mutex<Vec<ChromeTraceEvent>>>,
    start_time: Option<Instant>,
}

impl TraceResource {
    fn record(
        &self,
        name: &str,
        event_type: EventType,
        frame: u32,
        fixed_frame: u32,
        start: Instant,
        end: Instant,
    ) {
        let start_us = start.duration_since(self.start_time.unwrap()).as_nanos() as f64 / 1000.0;
        let dur_us = end.duration_since(start).as_nanos() as f64 / 1000.0;
        let thread_id = format!("{:?}", thread::current().id());

        let cat = match event_type {
            EventType::System => "System",
            EventType::Observer => "Observer",
            EventType::Frame => "Frame",
            EventType::FixedUpdateLoop => "FixedUpdate",
        }
        .to_string();

        let mut events = self.events.lock().unwrap();
        events.push(ChromeTraceEvent {
            name: name.to_string(),
            cat,
            ph: "X".to_string(),
            ts: start_us,
            dur: dur_us,
            pid: 1,
            tid: thread_id,
            args: serde_json::json!({
                "frame": frame,
                "fixed_frame": fixed_frame,
            }),
        });
    }
}

fn main() {
    let mut app = App::new();

    let fixed_update_timestep = Time::<Fixed>::default().timestep();
    let time_step = fixed_update_timestep * 2 + Duration::from_millis(1); // Run 2 FixedUpdates per frame

    let start_inst = Instant::now();
    app.add_plugins(MinimalPlugins)
        .insert_resource(TraceResource {
            events: Arc::new(Mutex::new(Vec::new())),
            start_time: Some(start_inst),
        })
        .init_resource::<FixedFrameCount>()
        .insert_resource(TimeUpdateStrategy::ManualDuration(time_step))
        // Schedules
        .add_systems(PreUpdate, pre_update)
        // Parallel systems in FixedUpdate
        .add_systems(
            FixedUpdate,
            (
                fixed_parallel_0,
                fixed_parallel_1,
                fixed_parallel_2,
                fixed_parallel_3,
            ),
        )
        .add_systems(FixedPostUpdate, fixed_post_update)
        .add_systems(FixedLast, increment_fixed_clock)
        .add_systems(Update, update)
        .add_systems(PostUpdate, post_update)
        // Observer Chain (Level 1 -> 2 -> 3 -> 4)
        .add_observer(on_level_1)
        .add_observer(on_level_2)
        .add_observer(on_level_3)
        .add_observer(on_level_4);

    for i in 0..3 {
        let start = Instant::now();
        app.update();
        let end = Instant::now();
        let trace = app.world().resource::<TraceResource>();
        trace.record(&format!("Frame {}", i), EventType::Frame, i, 0, start, end);
    }

    // Save trace to JSON
    let trace = app.world().resource::<TraceResource>();
    let events = trace.events.lock().unwrap();
    let json = serde_json::to_string_pretty(&*events).unwrap();
    let mut file = File::create("examples/ecs/trace.json").unwrap();
    file.write_all(json.as_bytes()).unwrap();
    println!("Trace saved to examples/ecs/trace.json");
}

// 还原为 1 2 3 4 索引名
#[derive(Event)]
struct Level1Event(&'static str);
#[derive(Event)]
struct Level2Event(&'static str);
#[derive(Event)]
struct Level3Event(&'static str);
#[derive(Event)]
struct Level4Event(&'static str);

#[derive(Resource, Default)]
pub struct FixedFrameCount(pub u32);

fn pre_update(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(100));
    commands.trigger(Level1Event("PreUpdate"));
    let end = Instant::now();
    trace.record("PreUpdate", EventType::System, frame.0, fixed.0, start, end);
}

fn fixed_parallel_0(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(300));
    commands.trigger(Level1Event("FixedParallel 0"));
    let end = Instant::now();
    trace.record(
        "FixedParallel 0",
        EventType::System,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn fixed_parallel_1(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(400));
    commands.trigger(Level1Event("FixedParallel 1"));
    let end = Instant::now();
    trace.record(
        "FixedParallel 1",
        EventType::System,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn fixed_parallel_2(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(500));
    commands.trigger(Level1Event("FixedParallel 2"));
    let end = Instant::now();
    trace.record(
        "FixedParallel 2",
        EventType::System,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn fixed_parallel_3(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(600));
    commands.trigger(Level1Event("FixedParallel 3"));
    let end = Instant::now();
    trace.record(
        "FixedParallel 3",
        EventType::System,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn on_level_1(
    ev: On<Level1Event>,
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    let tag = ev.event().0;
    thread::sleep(Duration::from_micros(50));
    commands.trigger(Level2Event(tag));
    let end = Instant::now();
    trace.record(
        &format!("Observer L1 ({})", tag),
        EventType::Observer,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn on_level_2(
    ev: On<Level2Event>,
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    let tag = ev.event().0;
    thread::sleep(Duration::from_micros(50));
    commands.trigger(Level3Event(tag));
    let end = Instant::now();
    trace.record(
        &format!("Observer L2 ({})", tag),
        EventType::Observer,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn on_level_3(
    ev: On<Level3Event>,
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    let tag = ev.event().0;
    thread::sleep(Duration::from_micros(50));
    commands.trigger(Level4Event(tag));
    let end = Instant::now();
    trace.record(
        &format!("Observer L3 ({})", tag),
        EventType::Observer,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn on_level_4(
    ev: On<Level4Event>,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    let tag = ev.event().0;
    thread::sleep(Duration::from_micros(50));
    let end = Instant::now();
    trace.record(
        &format!("Observer L4 ({})", tag),
        EventType::Observer,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn fixed_post_update(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(100));
    commands.trigger(Level1Event("FixedPostUpdate"));
    let end = Instant::now();
    trace.record(
        "FixedPostUpdate",
        EventType::System,
        frame.0,
        fixed.0,
        start,
        end,
    );
}

fn increment_fixed_clock(
    mut fixed_frame: ResMut<FixedFrameCount>,
    frame: Res<FrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(50));
    fixed_frame.0 += 1;
    let end = Instant::now();
    trace.record(
        "FixedLast",
        EventType::System,
        frame.0,
        fixed_frame.0,
        start,
        end,
    );
}

fn update(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(200));
    commands.trigger(Level1Event("Update"));
    let end = Instant::now();
    trace.record("Update", EventType::System, frame.0, fixed.0, start, end);
}

fn post_update(
    mut commands: Commands,
    frame: Res<FrameCount>,
    fixed: Res<FixedFrameCount>,
    trace: Res<TraceResource>,
) {
    let start = Instant::now();
    thread::sleep(Duration::from_micros(100));
    commands.trigger(Level1Event("PostUpdate"));
    let end = Instant::now();
    trace.record(
        "PostUpdate",
        EventType::System,
        frame.0,
        fixed.0,
        start,
        end,
    );
}
