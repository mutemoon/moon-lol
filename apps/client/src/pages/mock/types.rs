//! 离线 mock 页状态类型 + mock.json 数据移植。

use std::cell::RefCell;

use crate::components::agent_chat_history::AgentChatMessage;

/// mock.json 中 payload.agent_id / champion
pub(super) const AGENT_ID: &str = "riven_0";
pub(super) const CHAMPION: &str = "Riven";

// ── mock.json 数据移植 ──
// 原结构：{ event, payload: { agent_id, champion, history: [...] } }。
// history 每项 { role, content: [...] }，其中 content 里的块类型：
//   - { type: "text" }            → 用户提示 / 助手公开发言（message）
//   - assistant 嵌套 reasoning    → 思维链（think）
//   - { function: { name, arguments } } → 工具调用（tool_call）
//   - { type: "toolresult" }      → 工具结果（tool_call，role=user）
// 轮次按「决策周期第 N 次」的用户提示切分（第 1/2 轮）。

/// 移植自 mock.json 的初始决策流消息序列。
pub(super) fn initial_messages() -> Vec<AgentChatMessage> {
    fn m(
        role: &'static str,
        kind: &'static str,
        content: &'static str,
        round: u32,
    ) -> AgentChatMessage {
        AgentChatMessage {
            agent_id: AGENT_ID.to_string(),
            role: role.to_string(),
            kind: kind.to_string(),
            content: content.to_string(),
            round: Some(round),
        }
    }

    vec![
        // ── 决策周期 #1 ──
        m("user", "message", "这是你的决策周期第 1 次。请立刻调用 bash 工具运行 `cargo run --bin lol-cli -- --entity-id 4294965133 obs` 以获取最新局势观测，分析后做出行动命令（例如：`cargo run --bin lol-cli -- --entity-id 4294965133 act move --x 7500 --y 7500`）！", 1),
        m("assistant", "think", "用户要求我运行一个游戏客户端的命令行工具来获取局势观测并做出行动。让我先运行观测命令。", 1),
        m("assistant", "message", "好的，Riven玩家！让我立刻获取局势信息。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`cargo run --bin lol-cli -- --entity-id 4294965133 obs`", 1),
        m("user", "tool_call", "指令执行失败 (code=Some(101))：\nstdout: \nstderr: error: no bin target named `lol-cli` in default-run packages\nhelp: available bin targets:\n    moon-lol-desktop", 1),
        m("assistant", "think", "看起来这个项目没有 `lol-cli` bin target，而是有 `moon-lol-desktop`。但用户指定的命令是 `lol-cli`。让我检查一下项目结构，找出正确的二进制文件名称。", 1),
        m("assistant", "message", "让我先检查项目结构找到正确的命令。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`ls -la`", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`cat Cargo.toml 2>/dev/null || cat ../Cargo.toml 2>/dev/null`", 1),
        m("user", "tool_call", "错误: 安全策略限制，只允许执行以 lol_cli 开头的命令。", 1),
        m("assistant", "think", "看起来有安全策略限制，只允许执行以 `lol_cli` 开头的命令。也就是说，我的命令必须以 `lol_cli` 开头。所以正确的命令应该是 `lol_cli --entity-id 4294965133 obs` 之类的。让我试试看。", 1),
        m("assistant", "message", "好的，有安全策略限制，命令必须以 `lol_cli` 开头。让我重新尝试。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli --entity-id 4294965133 obs`", 1),
        m("user", "tool_call", "指令执行失败 (code=Some(2))：\nstdout: \nstderr: error: unexpected argument '--entity-id' found\n\nUsage: lol_cli [OPTIONS] <COMMAND>\n\nFor more information, try '--help'.", 1),
        m("assistant", "think", "看起来 `lol_cli` 这个命令需要先指定命令（如 `obs`），然后再传参数。或者可能需要用不同的格式。让我先看看帮助信息。", 1),
        m("assistant", "message", "让我看看帮助信息，了解正确的用法。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli --help`", 1),
        m("user", "tool_call", r#"命令行工具：自动连接当前游戏，供外部 Agent 对其进行观测与控制

Usage: lol_cli [OPTIONS] <COMMAND>

Commands:
  observe  获取当前英雄的局势观测数据 (Observe JSON)
  action   下达英雄动作指令
  pause    暂停游戏时间流速
  unpause  继续/恢复游戏时间流速
  state    获取调试服务当前的基本状态
  help     Print this message or the help of the given subcommand(s)

Options:
      --port <PORT>  游戏 WebSocket 服务的端口号 [default: 9001]
  -h, --help         Print help"#, 1),
        m("assistant", "think", "原来是这样！命令是：\n- `lol_cli observe` - 获取观测数据\n- `lol_cli action` - 下达动作指令\n\n不需要 `--entity-id` 参数，它可能是通过端口连接自动识别当前英雄的。让我先获取观测数据。", 1),
        m("assistant", "message", "明白了！通过 `observe` 获取观测，`action` 下达指令。先获取局势观测。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli observe`", 1),
        m("user", "tool_call", r#"```json
{
  "enemy_heroes": [
    { "distance": 1960.66, "entity": 4294965133, "health": 630.0, "max_health": 630.0, "position": [1981.0, 11441.0] }
  ],
  "minions": [],
  "myself": {
    "attack_damage": 66.0, "attack_range": 150.0, "attack_speed": 0.69,
    "gold": 506.56, "health": 620.0, "level": 1, "max_health": 620.0,
    "position": [3318.0, 12875.0], "skill_points": 1
  },
  "time": 3.22
}
```"#, 1),
        m("assistant", "think", r#"好的，让我分析目前的局势：

**我方（锐雯 - Riven）：** 位置 (3318, 12875)，等级 1，生命 620/620，攻击力 66，护甲 33，攻速 0.69，攻击范围 150（近战），金币 506，技能点 1（未分配）。

**敌方英雄（只有一个）：** 距离 1960.66 单位，生命 630/630，位置 (1981, 11441)。

**小兵：** 没有小兵信息（minions: []）。**时间：** 3.22 秒，游戏刚刚开始。

**分析：** 游戏刚开始 3 秒，作为锐雯 1 级通常学 Q（折翼之舞）。目前最需要做的是：
1. 先学一个技能（通常 1 级学 Q）
2. 走向兵线，准备补兵
3. 观察敌方走位

先看看 action 命令的用法。"#, 1),
        m("assistant", "message", "局势已获取！时间 3.22 秒，游戏刚开局。我是锐雯(Riven)，在蓝色方位置 (3318, 12875)，敌方英雄在 (1981, 11441) 距离约 1960 单位。让我看看行动命令的用法。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli action --help`", 1),
        m("user", "tool_call", r#"下达英雄动作指令

Usage: lol_cli action <COMMAND>

Commands:
  move            移动到指定坐标
  attack          攻击指定目标实体
  stop            停止所有动作
  skill           释放指定索引的技能到指定坐标
  skill-level-up  升级指定索引的技能
  help            Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help"#, 1),
        m("assistant", "think", r#"好的！可用的动作指令：move / attack / stop / skill / skill-level-up。

**锐雯对线思路：** 1 级通常学 **Q（折翼之舞）** - 技能索引 0，三段位移+伤害；也可以学 **E（勇往直前）** - 技能索引 2，护盾+位移。

**当前局势：** 位置 (3318, 12875)，敌方 (1981, 11441)，距离约 1960 单位，没有小兵信息，金币 506。

**第一步行动方案：**
1. 先升级 Q 技能（技能索引 0）
2. 向兵线方向移动准备补刀"#, 1),
        m("assistant", "message", r#"好的！现在我来执行锐雯的 1 级对线策略。先升级 Q 技能（折翼之舞），然后向兵线方向移动准备补刀。

**分析局势：**
- 🕐 游戏时间 3.22 秒，刚开局
- 🏆 我方锐雯：620HP，66 攻击力，1 级，技能点未分配
- 👤 敌方英雄：630HP，在 (1981, 11441)，距我约 1960 码
- 📍 我当前位置：(3318, 12875)

**决策：** 1 级学 Q（技能索引 0），向兵线交汇处移动准备补刀。"#, 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli action skill-level-up --index 0`", 1),
        m("user", "tool_call", "指令执行失败 (code=Some(2))：\nstdout: \nstderr: error: unexpected argument '--index' found\n\n  tip: to pass '--index' as a value, use '-- --index'\n\nUsage: lol_cli action skill-level-up <INDEX>\n\nFor more information, try '--help'.", 1),
        m("assistant", "think", "看起来 `skill-level-up` 命令的参数是位置参数，不是 `--index`。应该是 `lol_cli action skill-level-up 0`。", 1),
        m("assistant", "message", "参数是位置参数，直接传技能索引。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli action skill-level-up 0`", 1),
        m("user", "tool_call", r#"```json
{
  "status": "success"
}
```"#, 1),
        m("assistant", "think", r#"Q 技能升级成功！现在锐雯的 Q 技能（折翼之舞）已经升到 1 级。

接下来我需要向兵线方向移动。我的位置是 (3318, 12875)，敌方在 (1981, 11441)。我打算先往中间靠拢，大概是 (2800, 12150) 左右的位置，这样可以兼顾补兵和安全。"#, 1),
        m("assistant", "message", "✅ Q 技能（折翼之舞）已升级！现在向兵线方向移动，准备对线补刀。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli action move --x 2800 --y 12200`", 1),
        m("user", "tool_call", "指令执行失败 (code=Some(2))：\nstdout: \nstderr: error: unexpected argument '--x' found\n\n  tip: to pass '--x' as a value, use '-- --x'\n\nUsage: lol_cli action move <X> <Y>\n\nFor more information, try '--help'.", 1),
        m("assistant", "think", "又是位置参数，不是 `--x` 和 `--y`。应该是 `lol_cli action move 2800 12200`。", 1),
        m("assistant", "message", "参数也是位置参数，直接传坐标值。", 1),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli action move 2800 12200`", 1),
        m("user", "tool_call", r#"```json
{
  "status": "success"
}
```"#, 1),
        m("assistant", "think", r#"移动指令成功！现在锐雯正在向 (2800, 12200) 移动。

**这个周期完成的操作：**
1. ✅ 获取局势观测 - 了解双方状态和位置
2. ✅ 升级 Q 技能（折翼之舞）- 技能索引 0
3. ✅ 向兵线方向移动至 (2800, 12200)

**下一步计划（下个决策周期）：** 再次观测局势，小兵到达后开始补刀，保持安全距离。"#, 1),
        m("assistant", "message", r#"## ✅ 决策周期 #1 完成

### 执行操作总结

| 操作 | 状态 | 说明 |
|------|:----:|:-----|
| 🔍 获取局势观测 | ✅ | 时间 3.22s，锐雯 1 级 620HP，敌方 630HP 相距 1960 码 |
| ⬆️ 升级 Q 技能（折翼之舞） | ✅ | 技能索引 0 → 等级 1 |
| 🚶 向兵线移动 | ✅ | 目标坐标 (2800, 12200) |

### 当前对线策略

作为 **锐雯** 玩家，1 级对线思路如下：

1. **Q 技能（折翼之舞）** 已就绪 — 三段位移+伤害，配合被动 **符文之刃**（技能后普攻强化）打出爆发
2. **走位意识**：向兵线区域移动，等待小兵到达
3. **补刀准备**：利用 Q 的伤害和普攻补刀，积累经济优势
4. **换血时机**：观察敌方技能释放，利用 Q 的灵活性和 E（2 级学）的护盾进行无伤换血

> ⚔️ **锐雯箴言**：*"断剑重铸之日，骑士归来之时！"* — 耐心补刀发育，等待斩杀线的到来。"#, 1),

        // ── 决策周期 #2 ──
        m("user", "message", "这是你的决策周期第 2 次。请立刻调用 bash 工具运行 `cargo run --bin lol-cli -- --entity-id 4294965133 obs` 以获取最新局势观测，分析后做出行动命令（例如：`cargo run --bin lol-cli -- --entity-id 4294965133 act move --x 7500 --y 7500`）！", 2),
        m("assistant", "think", "用户再次要求获取局势观测。但根据之前的经验，正确的命令是 `lol_cli observe` 而不是 `cargo run --bin lol-cli`。而且安全策略限制只允许执行以 `lol_cli` 开头的命令。让我直接运行 `lol_cli observe` 获取最新局势。", 2),
        m("assistant", "message", "好的，锐雯！第 2 个决策周期，立刻获取最新局势！", 2),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli observe`", 2),
        m("user", "tool_call", r#"```json
{
  "enemy_heroes": [
    { "distance": 1224.2, "entity": 4294965132, "health": 620.0, "max_health": 620.0, "position": [3091.5, 12579.9] }
  ],
  "minions": [],
  "myself": {
    "attack_damage": 64.0, "gold": 508.76, "health": 630.0, "level": 1,
    "max_health": 630.0, "position": [2235.1, 11705.1],
    "run_target": [3000.0, 12500.0], "skill_points": 0
  },
  "time": 4.305
}
```"#, 2),
        m("assistant", "think", r#"好的，让我分析最新局势：

**时间：** 4.30 秒。**我方锐雯：** 位置 (2235, 11705)，生命 630/630，攻击力 64，金币 508.76，Q 技能 1 级。**敌方英雄：** entity 4294965132，生命 620/620，位置 (3091, 12579)，距离 1224。

**分析：** 我从 (3318, 12875) 移动到 (2235, 11705)，敌方从 (1981, 11441) 移动到 (3091, 12579)，双方距离从 1960 缩小到 1224，正在相向而行。小兵尚未到达。

继续向兵线交汇处移动，占据有利补刀位置。"#, 2),
        m("assistant", "message", r#"局势更新！时间 4.30 秒，我正在向兵线移动中。让我分析最新情况：

**📊 局势变化：**
| 指标 | 上次(3.22s) | 本次(4.30s) | 变化 |
|:----|:-----------:|:-----------:|:----:|
| 我的位置 | (3318, 12875) | **(2235, 11705)** | ⬇️ 向敌方靠近 |
| 敌方位置 | (1981, 11441) | **(3091, 12579)** | ⬆️ 向我方靠近 |
| 双方距离 | 1960 码 | **1224 码** | 📏 缩短 736 码 |
| 小兵 | 无 | 无 | ⏳ 尚未到达 |

**分析：** 双方正在相向而行，距离缩短到 1224 码。Q 技能已就绪，小兵即将到达线上。继续向兵线交汇处移动，占据有利补刀位置！"#, 2),
        m("assistant", "tool_call", "调用 bash 工具：\n`lol_cli action move 2700 12300`", 2),
        m("user", "tool_call", r#"```json
{
  "status": "success"
}
```"#, 2),
        m("assistant", "think", "移动指令成功，现在锐雯正在向 (2700, 12300) 移动。这个位置更接近兵线交汇处，方便我补刀和对线。让我总结一下这个决策周期。", 2),
        m("assistant", "message", r#"✅ 移动指令已下达！目标位置 (2700, 12300)。

---

## ✅ 决策周期 #2 完成

### 执行操作
| 操作 | 状态 | 说明 |
|:----|:----:|:------|
| 🔍 获取局势观测 | ✅ | 时间 4.30s，双方距离缩短至 1224 码 |
| 🚶 调整移动目标 | ✅ | (2235,11705) → **(2700, 12300)** 兵线中心区域 |

### 🎯 锐雯对线策略分析

**当前处境：** 等级 1，Q 技能就绪，满血 630/630，敌方 620HP，距我 1224 码，小兵尚未到达线上。

**下周期预判：**
1. ⏳ **小兵即将到达** — 准备利用 **Q+被动平A** 补刀
2. 🛡️ **保持距离** — 1224 码是比较安全的距离
3. 👀 **观察敌方技能** — 注意敌方英雄的起手动作，准备用 Q 灵活躲避
4. 💰 **补刀优先** — 先稳健发育到 2 级，学 E(勇往直前) 后就可以 **QAEA** 无伤换血

> ⚔️ *"真正的战士，生于鲜血，死于荣耀。"* — 稳住发育，等待 3 级斩杀线的到来！"#, 2),
    ]
}

// ── 页面本地状态 ──

/// 页面视图：列表（index.vue 落地页）与会话（chat.vue 调试床）。
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum MockView {
    List,
    Chat,
}

pub(super) struct MockPageState {
    pub(super) view: MockView,
    pub(super) messages: Vec<AgentChatMessage>,
    pub(super) user_input: String,
    pub(super) assistant_input: String,
}

impl Default for MockPageState {
    fn default() -> Self {
        Self {
            view: MockView::List,
            messages: initial_messages(),
            user_input: String::new(),
            assistant_input: String::new(),
        }
    }
}

thread_local! {
    static STATE: RefCell<MockPageState> = RefCell::new(MockPageState::default());
}

pub(super) fn with_state<R>(f: impl FnOnce(&MockPageState) -> R) -> R {
    STATE.with(|s| f(&s.borrow()))
}

pub(super) fn update_state(f: impl FnOnce(&mut MockPageState)) {
    STATE.with(|s| f(&mut s.borrow_mut()));
}

/// 当前最大轮次（作为新追加消息的归属轮次）。
pub(super) fn current_round(messages: &[AgentChatMessage]) -> u32 {
    messages.iter().filter_map(|m| m.round).max().unwrap_or(1)
}
