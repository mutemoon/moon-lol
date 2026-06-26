//! 对局运行时的决策驱动分发层（产品文档 §3 / arch.md §三）。
//!
//! 三类 Agent 共享统一的 [`AgentDriver`] 契约：
//!   - [`LlmDriver`]：现有 WebSocket 观测/动作桥（决策在外部 LLM 执行器，见 `systems.rs`）；
//!   - [`RlDriver`]：RL 推理端点适配（占位，决策在外部 Gym/推理服务）；
//!   - [`ScriptDriver`]：在 Rust 侧用 `rquickjs` 内嵌 JS 沙盒运行用户脚本。
//!
//! `ScriptDriver` 的安全要点：
//!   - **沙盒**：QuickJS 默认不暴露任何文件/网络 I/O，只注入我们显式绑定的宿主函数；
//!   - **时间片熔断**：通过 QuickJS 中断回调限制单 tick 的 CPU 时长，死循环会被强制中断而非挂起引擎；
//!   - **热重载**：脚本源码可随时替换，`globalThis.state` 跨 tick 持久，重载不丢状态。

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use bevy::prelude::{Component, Entity, Vec2};
use lol_core::action::Action;
use rquickjs::{Context, Ctx, Function, Runtime};
use serde::Deserialize;

use crate::models::Observe;

/// 单 tick 默认 CPU 预算（超出即被中断回调强制熔断）。
pub const DEFAULT_TICK_BUDGET: Duration = Duration::from_millis(5);

/// 标记一个由 JS 脚本驱动的对局实体，并携带脚本源码。
/// 修改 `source`（变更检测）即触发运行时热重载，保留脚本状态。
#[derive(Component, Clone, Debug)]
pub struct ScriptAgent {
    pub source: String,
}

/// 对局内所有 Script 驱动的持有者。`ScriptDriver` 非 `Send`，故以 `NonSend` 资源存放。
#[derive(Default)]
pub struct ScriptRuntimes(pub HashMap<Entity, ScriptDriver>);

/// Agent 决策驱动类型。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentKind {
    Llm,
    Rl,
    Script,
}

impl AgentKind {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "llm" => Some(AgentKind::Llm),
            "rl" => Some(AgentKind::Rl),
            "script" => Some(AgentKind::Script),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AgentKind::Llm => "llm",
            AgentKind::Rl => "rl",
            AgentKind::Script => "script",
        }
    }
}

/// 统一的决策驱动契约：接收一帧观测、产出本帧要下发的 ECS 动作。
pub trait AgentDriver {
    /// 驱动类型。
    fn kind(&self) -> AgentKind;
    /// 接收一帧局势观测（[`Observe`]）。
    fn observe(&mut self, observe: &Observe);
    /// 取出本帧决策出的动作（下发到 ECS 行动队列）。
    fn actions(&mut self) -> Vec<Action>;
    /// 运行时热重载策略（仅 Script 有意义；其余默认空实现）。
    fn reload(&mut self, _source: &str) {}
    /// 取出脚本/驱动产生的日志（供调试面板展示）。
    fn take_logs(&mut self) -> Vec<String> {
        Vec::new()
    }
    /// 上一次执行的错误（编译/运行/熔断），无则 None。
    fn last_error(&self) -> Option<&str> {
        None
    }
}

/// 按类型实例化驱动。Bevy 开局时据此为每个 Agent 选择驱动实现。
pub fn create_driver(
    kind: AgentKind,
    script_source: Option<&str>,
) -> Result<Box<dyn AgentDriver>, String> {
    match kind {
        AgentKind::Llm => Ok(Box::new(LlmDriver::default())),
        AgentKind::Rl => Ok(Box::new(RlDriver::default())),
        AgentKind::Script => {
            let mut driver = ScriptDriver::new(DEFAULT_TICK_BUDGET)?;
            if let Some(src) = script_source {
                driver.reload(src);
            }
            Ok(Box::new(driver))
        }
    }
}

// ════════════════════════ LLM / RL 占位驱动 ════════════════════════

/// LLM 驱动：实际决策由外部 LLM 执行器经 WebSocket 完成（见 `systems.rs` 的
/// `on_command_ws_request`），此处仅作类型分发占位，不在引擎内同步推理。
#[derive(Default)]
pub struct LlmDriver;

impl AgentDriver for LlmDriver {
    fn kind(&self) -> AgentKind {
        AgentKind::Llm
    }
    fn observe(&mut self, _observe: &Observe) {}
    fn actions(&mut self) -> Vec<Action> {
        Vec::new()
    }
}

/// RL 驱动：实际推理由外部 Gym/推理端点完成（高频张量 WS，后续接入），
/// 此处作类型分发占位。
#[derive(Default)]
pub struct RlDriver;

impl AgentDriver for RlDriver {
    fn kind(&self) -> AgentKind {
        AgentKind::Rl
    }
    fn observe(&mut self, _observe: &Observe) {}
    fn actions(&mut self) -> Vec<Action> {
        Vec::new()
    }
}

// ════════════════════════ Script 驱动（rquickjs 沙盒） ════════════════════════

/// 宿主与 JS 之间共享的缓冲：观测输入、动作/日志输出、wait 计数。
#[derive(Default)]
struct Shared {
    observe_json: String,
    actions: Vec<String>,
    logs: Vec<String>,
    wait: i32,
}

/// 注入到沙盒的 JS 前导：把底层原生绑定（`__*`）包装成对脚本友好的
/// `observe()` / `action()` / `log()` / `wait_ticks()`，并初始化跨 tick 持久的 `state`。
const PRELUDE: &str = r#"
globalThis.state = globalThis.state || {};
function observe() { return JSON.parse(__observe()); }
function action(a) { __push_action(JSON.stringify(a)); }
function log() {
  var parts = [];
  for (var i = 0; i < arguments.length; i++) {
    var x = arguments[i];
    parts.push(typeof x === 'object' && x !== null ? JSON.stringify(x) : String(x));
  }
  __log(parts.join(' '));
}
function wait_ticks(n) { __wait(n | 0); }
"#;

/// 脚本动作的反序列化镜像（与 `lol_core::action::Action` 的 serde 外部标签一致），
/// 实体用 `u64`（与 `Entity::to_bits()` 对应），坐标用 `[f32; 2]`，避免依赖 glam/Entity 的具体序列化形态。
#[derive(Deserialize)]
enum ScriptAction {
    Stop,
    Attack(u64),
    Move([f32; 2]),
    Skill { index: usize, point: [f32; 2] },
    SkillLevelUp(usize),
}

impl ScriptAction {
    fn into_action(self) -> Action {
        match self {
            ScriptAction::Stop => Action::Stop,
            ScriptAction::Attack(bits) => Action::Attack(Entity::from_bits(bits)),
            ScriptAction::Move([x, z]) => Action::Move(Vec2::new(x, z)),
            ScriptAction::Skill {
                index,
                point: [x, z],
            } => Action::Skill {
                index,
                point: Vec2::new(x, z),
            },
            ScriptAction::SkillLevelUp(index) => Action::SkillLevelUp(index),
        }
    }
}

/// 在 Rust 侧用 `rquickjs` 运行用户 JS 脚本的决策驱动。
///
/// 非 `Send`（QuickJS 运行时绑定单线程）；在 Bevy 中以 `NonSend` 资源持有。
pub struct ScriptDriver {
    // 注意：Context 持有对 Runtime 的引用计数，二者需一同存活，故同时保留。
    _runtime: Runtime,
    context: Context,
    shared: Rc<RefCell<Shared>>,
    /// 每个 tick 开始时间，供中断回调判断是否超时熔断。
    start: Rc<Cell<Instant>>,
    source: String,
    last_error: Option<String>,
    /// wait_ticks(n) 设置的剩余跳过 tick 数（协作式让出）。
    skip_ticks: u32,
    last_reload: Option<Instant>,
}

impl ScriptDriver {
    /// 创建一个空脚本的驱动，单 tick CPU 预算为 `budget`。
    pub fn new(budget: Duration) -> Result<Self, String> {
        let runtime = Runtime::new().map_err(|e| e.to_string())?;

        // 时间片熔断：QuickJS 在执行字节码时周期性调用该回调，返回 true 即中断当前执行。
        let start = Rc::new(Cell::new(Instant::now()));
        let start_for_handler = start.clone();
        runtime.set_interrupt_handler(Some(Box::new(move || {
            start_for_handler.get().elapsed() >= budget
        })));

        let context = Context::full(&runtime).map_err(|e| e.to_string())?;
        let shared = Rc::new(RefCell::new(Shared::default()));

        let install = {
            let shared = shared.clone();
            context.with(|ctx| -> rquickjs::Result<()> {
                let g = ctx.globals();

                let s = shared.clone();
                g.set(
                    "__observe",
                    Function::new(ctx.clone(), move || -> String {
                        s.borrow().observe_json.clone()
                    })?,
                )?;

                let s = shared.clone();
                g.set(
                    "__push_action",
                    Function::new(ctx.clone(), move |a: String| {
                        s.borrow_mut().actions.push(a);
                    })?,
                )?;

                let s = shared.clone();
                g.set(
                    "__log",
                    Function::new(ctx.clone(), move |m: String| {
                        s.borrow_mut().logs.push(m);
                    })?,
                )?;

                let s = shared.clone();
                g.set(
                    "__wait",
                    Function::new(ctx.clone(), move |n: i32| {
                        s.borrow_mut().wait = n.max(0);
                    })?,
                )?;

                ctx.eval::<(), _>(PRELUDE.as_bytes().to_vec())?;
                Ok(())
            })
        };
        install.map_err(|e| format!("初始化 JS 沙盒失败: {e}"))?;

        Ok(Self {
            _runtime: runtime,
            context,
            shared,
            start,
            source: String::new(),
            last_error: None,
            skip_ticks: 0,
            last_reload: None,
        })
    }

    /// 上次热重载时间。
    pub fn last_reload(&self) -> Option<Instant> {
        self.last_reload
    }

    /// 以给定观测 JSON 跑一次脚本（设置观测 -> 执行 -> 收集动作/日志/wait）。
    fn run(&mut self, observe_json: &str) {
        self.last_error = None;

        // 协作式让出：wait_ticks 期间跳过执行。
        if self.skip_ticks > 0 {
            self.skip_ticks -= 1;
            return;
        }

        {
            let mut sh = self.shared.borrow_mut();
            sh.observe_json = observe_json.to_string();
            sh.actions.clear();
            sh.wait = 0;
        }

        // 复位时间片起点，随后执行用户脚本。
        self.start.set(Instant::now());
        let src = self.source.clone();
        let result = self.context.with(|ctx| -> Result<(), String> {
            ctx.eval::<(), _>(src.into_bytes())
                .map_err(|e| describe_error(&ctx, e))
        });
        if let Err(msg) = result {
            self.last_error = Some(msg);
        }

        let wait = self.shared.borrow().wait;
        if wait > 0 {
            self.skip_ticks = wait as u32;
        }
    }

    /// 取出并清空本帧脚本产生的动作。
    fn take_actions(&mut self) -> Vec<Action> {
        let raw = {
            let mut sh = self.shared.borrow_mut();
            std::mem::take(&mut sh.actions)
        };
        raw.into_iter()
            .filter_map(|s| serde_json::from_str::<ScriptAction>(&s).ok())
            .map(ScriptAction::into_action)
            .collect()
    }
}

impl AgentDriver for ScriptDriver {
    fn kind(&self) -> AgentKind {
        AgentKind::Script
    }

    fn observe(&mut self, observe: &Observe) {
        match serde_json::to_string(observe) {
            Ok(json) => self.run(&json),
            Err(e) => self.last_error = Some(format!("观测序列化失败: {e}")),
        }
    }

    fn actions(&mut self) -> Vec<Action> {
        self.take_actions()
    }

    fn reload(&mut self, source: &str) {
        // 仅替换源码，`globalThis.state` 在同一 Context 中跨重载持久。
        self.source = source.to_string();
        self.last_reload = Some(Instant::now());
    }

    fn take_logs(&mut self) -> Vec<String> {
        let mut sh = self.shared.borrow_mut();
        std::mem::take(&mut sh.logs)
    }

    fn last_error(&self) -> Option<&str> {
        self.last_error.as_deref()
    }
}

/// 把 rquickjs 的执行错误转成可读字符串（尽量取异常 message）。
fn describe_error(ctx: &Ctx, err: rquickjs::Error) -> String {
    if err.is_exception() {
        let caught = ctx.catch();
        if let Some(ex) = caught.as_exception() {
            if let Some(msg) = ex.message() {
                return msg;
            }
            return ex.to_string();
        }
        return "脚本抛出异常（含时间片熔断中断）".to_string();
    }
    err.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn driver(source: &str) -> ScriptDriver {
        // 测试用更宽的预算，避免 CI 抖动误熔断。
        let mut d = ScriptDriver::new(Duration::from_millis(200)).unwrap();
        d.reload(source);
        d
    }

    fn empty_observe() -> Observe {
        use crate::models::ObserveMyself;
        Observe {
            time: 1.0,
            myself: ObserveMyself {
                position: Vec2::new(1.0, 2.0),
                attack_state: None,
                run_target: None,
                health: 100.0,
                max_health: 100.0,
                level: 1,
                ability_resource: None,
                attack_damage: 60.0,
                attack_range: 175.0,
                attack_speed: 0.6,
                armor: 30.0,
                skill_points: 1,
                skills: Vec::new(),
                gold: 0.0,
                kills: 0,
                deaths: 0,
                assists: 0,
                minion_kills: 0,
            },
            minions: Vec::new(),
            friendly_heroes: Vec::new(),
            enemy_heroes: Vec::new(),
        }
    }

    #[test]
    fn kinds_round_trip() {
        for k in [AgentKind::Llm, AgentKind::Rl, AgentKind::Script] {
            assert_eq!(AgentKind::from_str(k.as_str()), Some(k));
        }
        assert_eq!(AgentKind::from_str("nope"), None);
    }

    #[test]
    fn factory_builds_each_kind() {
        assert_eq!(
            create_driver(AgentKind::Llm, None).unwrap().kind(),
            AgentKind::Llm
        );
        assert_eq!(
            create_driver(AgentKind::Rl, None).unwrap().kind(),
            AgentKind::Rl
        );
        let d = create_driver(AgentKind::Script, Some("action('Stop');")).unwrap();
        assert_eq!(d.kind(), AgentKind::Script);
    }

    #[test]
    fn script_emits_stop_action() {
        let mut d = driver("action('Stop');");
        d.observe(&empty_observe());
        let actions = d.actions();
        assert_eq!(actions.len(), 1);
        assert!(matches!(actions[0], Action::Stop));
        assert!(d.last_error().is_none());
    }

    #[test]
    fn script_can_read_observe_and_move() {
        // 读取自身坐标并据此 Move，验证 observe() 宿主绑定贯通。
        let mut d = driver(
            "var o = observe(); action({ Move: [o.myself.position[0] + 10, o.myself.position[1]] });",
        );
        d.observe(&empty_observe());
        let actions = d.actions();
        assert_eq!(actions.len(), 1);
        match actions[0] {
            Action::Move(p) => {
                assert!((p.x - 11.0).abs() < 1e-3);
                assert!((p.y - 2.0).abs() < 1e-3);
            }
            _ => panic!("expected Move, got {:?}", actions[0]),
        }
    }

    #[test]
    fn script_parses_skill_and_attack() {
        let mut d = driver(
            "action({ Skill: { index: 0, point: [5, 6] } }); action({ Attack: 42 }); action({ SkillLevelUp: 1 });",
        );
        d.observe(&empty_observe());
        let actions = d.actions();
        assert_eq!(actions.len(), 3);
        assert!(matches!(actions[0], Action::Skill { index: 0, .. }));
        assert!(matches!(actions[1], Action::Attack(_)));
        assert!(matches!(actions[2], Action::SkillLevelUp(1)));
    }

    #[test]
    fn log_is_captured() {
        let mut d = driver("log('hello', 42);");
        d.observe(&empty_observe());
        let logs = d.take_logs();
        assert_eq!(logs, vec!["hello 42".to_string()]);
    }

    #[test]
    fn wait_ticks_skips_execution() {
        // 第 1 tick 计数并 wait_ticks(2)，随后 2 个 tick 应被跳过，第 4 个 tick 再次执行。
        let mut d = driver("state.n = (state.n || 0) + 1; log(state.n); wait_ticks(2);");
        for _ in 0..4 {
            d.observe(&empty_observe());
        }
        let logs = d.take_logs();
        // tick1 执行(n=1) -> tick2 跳过 -> tick3 跳过 -> tick4 执行(n=2)
        assert_eq!(logs, vec!["1".to_string(), "2".to_string()]);
    }

    #[test]
    fn time_slice_fuse_aborts_infinite_loop() {
        // 死循环必须被熔断而非挂起：tick 返回后 last_error 应有值。
        let mut d = ScriptDriver::new(Duration::from_millis(30)).unwrap();
        d.reload("while (true) {}");
        d.observe(&empty_observe());
        assert!(
            d.last_error().is_some(),
            "infinite loop should be interrupted"
        );
    }

    #[test]
    fn hot_reload_preserves_state() {
        let mut d = driver("state.n = (state.n || 0) + 1; log(state.n);");
        d.observe(&empty_observe()); // n = 1
        d.observe(&empty_observe()); // n = 2
        // 热重载到新脚本，state 应保留。
        d.reload("log('reloaded ' + state.n);");
        d.observe(&empty_observe());
        let logs = d.take_logs();
        assert_eq!(
            logs,
            vec!["1".to_string(), "2".to_string(), "reloaded 2".to_string()]
        );
        assert!(d.last_reload().is_some());
    }

    #[test]
    fn syntax_error_surfaces_without_panic() {
        let mut d = driver("this is not valid js );");
        d.observe(&empty_observe());
        assert!(d.last_error().is_some());
        assert!(d.actions().is_empty());
    }
}
