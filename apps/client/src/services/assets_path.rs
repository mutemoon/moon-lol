use std::path::PathBuf;

/// 动态解析 assets 资源根目录：
/// 1. 开发模式（CARGO_MANIFEST_DIR 存在）：向上递归寻找含有 rust-toolchain.toml 或 pnpm-workspace.yaml 的 workspace 根目录下的 `assets` 文件夹。
/// 2. 发布模式（Release 构建）：取 `client.exe` 同级目录下的 `assets` 文件夹。
/// 3. Fallback：当前工作目录下的 `assets`。
pub fn resolve_assets_dir() -> PathBuf {
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest);
        loop {
            if path.join("rust-toolchain.toml").exists()
                || path.join("pnpm-workspace.yaml").exists()
            {
                return path.join("assets");
            }
            if let Some(parent) = path.parent() {
                path = parent.to_path_buf();
            } else {
                break;
            }
        }
    }

    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(exe_dir) = exe_path.parent() {
            return exe_dir.join("assets");
        }
    }

    PathBuf::from("assets")
}
