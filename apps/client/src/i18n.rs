use std::fs;
use std::path::PathBuf;

/// 持久化语言文件路径：Windows 用 %APPDATA%/moon-lol/locale，否则当前目录 .moon-lol/locale
fn locale_file() -> PathBuf {
    if let Ok(appdata) = std::env::var("APPDATA") {
        PathBuf::from(appdata).join("moon-lol").join("locale")
    } else {
        PathBuf::from(".moon-lol").join("locale")
    }
}

pub const DEFAULT_LOCALE: &str = "zh-CN";
pub const AVAILABLE_LOCALES: [&str; 2] = ["zh-CN", "en"];

pub fn read_persisted_locale() -> String {
    match fs::read_to_string(locale_file()) {
        Ok(v) if AVAILABLE_LOCALES.contains(&v.trim()) => v.trim().to_string(),
        _ => DEFAULT_LOCALE.to_string(),
    }
}

pub fn persist_locale(locale: &str) {
    let path = locale_file();
    let _ = fs::create_dir_all(path.parent().unwrap_or(&path));
    let _ = fs::write(path, locale);
}
