use std::path::PathBuf;

pub fn model_root() -> PathBuf {
    if let Ok(dir) = std::env::var("MOON_LOL_HOME") {
        return PathBuf::from(dir);
    }
    #[cfg(target_os = "windows")]
    {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| {
                std::env::var("HOMEDRIVE")
                    .and_then(|d| std::env::var("HOMEPATH").map(|p| format!("{d}{p}")))
            })
            .unwrap_or_default();
        PathBuf::from(home).join(".moon-lol")
    }
    #[cfg(not(target_os = "windows"))]
    {
        let home = std::env::var("HOME").unwrap_or_default();
        PathBuf::from(home).join(".moon-lol")
    }
}

pub fn checkpoint_dir(task_id: &str) -> PathBuf {
    model_root().join("checkpoints").join(task_id)
}

pub fn new_checkpoint_path(task_id: &str, ckpt_id: &str) -> PathBuf {
    checkpoint_dir(task_id).join(format!("{ckpt_id}.safetensors"))
}
