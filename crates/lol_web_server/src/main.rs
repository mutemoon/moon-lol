// 主入口：旧的单体 handlers/services 正在被新分层取代。
// 迁移期间此入口暂时简化；待 handler 层迁移完成后恢复 Axum server 启动。

fn main() {
    eprintln!("lol_web_server: handler 层迁移中，暂时无法启动。");
    eprintln!("库模块（domain/repository/cache/service）已就绪，可用 `cargo test` 验证。");
}
