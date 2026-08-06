//! 全局服务定位器：懒初始化单例，供页面层访问 ProcessService 与 CloudClient。

use std::sync::OnceLock;

use super::cloud::CloudClient;
use super::process_service::ProcessService;

pub fn process_service() -> &'static ProcessService {
    static SVC: OnceLock<ProcessService> = OnceLock::new();
    SVC.get_or_init(ProcessService::new)
}

pub fn cloud_client() -> &'static CloudClient {
    static SVC: OnceLock<CloudClient> = OnceLock::new();
    SVC.get_or_init(|| CloudClient::new(None))
}
