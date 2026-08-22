use std::path::PathBuf;

pub fn model_root() -> PathBuf {
    lol_share::paths::moon_home_dir()
}

pub fn checkpoint_dir(task_id: &str) -> PathBuf {
    lol_share::paths::checkpoint_task_dir(task_id)
}

pub fn new_checkpoint_path(task_id: &str, ckpt_id: &str) -> PathBuf {
    lol_share::paths::checkpoint_path(task_id, ckpt_id)
}

