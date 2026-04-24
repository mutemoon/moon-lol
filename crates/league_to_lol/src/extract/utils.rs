use serde::Serialize;

/// 将内容写入文件，自动创建父目录
pub fn write_to_file(path: &str, content: impl AsRef<[u8]>) {
    let path = std::path::Path::new(path);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).expect("无法创建目录");
    }
    std::fs::write(path, content).expect("无法写入文件");
}

/// 将二进制序列化内容写入文件
pub fn write_bin_to_file<T: Serialize>(path: &str, content: &T) {
    write_to_file(
        path,
        bincode::serialize(content).expect("无法序列化为二进制"),
    );
}
