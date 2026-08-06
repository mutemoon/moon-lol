use std::collections::VecDeque;

/// 应用层事件枚举 —— 服务层发布，视图/状态层订阅处理。
/// M1 阶段定义框架，M3/M4 填充具体事件与调度逻辑。
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// WS 训练服务连接状态变更
    WsConnected,
    WsDisconnected,
    /// 可视化子进程 WS 连接状态变更
    VisualWsConnected,
    VisualWsDisconnected,
    /// RL 训练任务列表已刷新
    TaskListRefreshed,
    /// 指定任务的详情已更新
    TaskDetailUpdated(String),
    /// 对局进程启动
    GameStarted(String),
    /// 对局进程停止
    GameStopped(String),
    /// 对局列表变更
    GameListUpdated,
    /// 认证状态变更（token / 用户信息更新后）
    AuthStateChanged,
    /// 模型供应商列表已更新
    ProvidersUpdated,
}

/// 事件发布 trait —— 服务实现此 trait 可向 AppSidebar 推送事件。
/// M1 阶段为轻量抽象；实际调度通过 entity.update() 完成。
pub trait EventDispatcher {
    fn dispatch(&self, event: AppEvent);
}

/// 简易内存事件队列，用于延迟消费或批量处理。
pub struct EventQueue {
    events: VecDeque<AppEvent>,
    capacity: usize,
}

impl EventQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn push(&mut self, event: AppEvent) {
        if self.events.len() >= self.capacity {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    pub fn drain(&mut self) -> Vec<AppEvent> {
        self.events.drain(..).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }
}

impl EventDispatcher for EventQueue {
    fn dispatch(&self, event: AppEvent) {
        // EventQueue 作为 EventDispatcher 实现：将事件入队。
        // 注意：这里需要 &mut self，但 trait 定义为 &self。
        // M1 阶段提供默认空实现，调度由 AppSidebar 上的方法完成。
        let _ = event;
    }
}
