//! 设置页状态类型（存储在 `AppSidebar.settings`）。

use lol_web_protocol::model_provider::{ModelConfig, ModelProvider, TestModelProviderResponse};

#[derive(Clone, Copy, PartialEq)]
pub enum SettingsTab {
    General,
    ModelSettings,
}

pub(super) const PLATFORM_KEY: &str = "__platform__";
pub(super) const NEW_KEY: &str = "__new__";
pub(super) const PRESET_PREFIX: &str = "__preset__:";

pub struct SettingsState {
    pub active_tab: SettingsTab,
    pub providers: Vec<ModelProvider>,
    pub loading: bool,
    pub error_msg: String,
    pub success_msg: String,

    pub selected_key: String,
    pub form_name: String,
    pub form_base_url: String,
    pub form_api_key: String,
    pub form_api_format: String,
    pub form_models: Vec<ModelConfig>,
    pub form_has_api_key: bool,
    pub form_category: String,
    pub form_preset_type: String,
    pub form_website_url: String,
    pub form_api_key_url: String,
    pub form_icon: String,
    pub form_icon_color: String,
    pub form_sort_order: i32,
    pub saving: bool,

    pub show_model_dialog: bool,
    pub editing_model_idx: Option<usize>,
    pub model_form_name: String,
    pub model_form_max_tokens: String,

    pub testing_model_idx: Option<usize>,
    pub test_result: Option<TestModelProviderResponse>,
    pub show_test_result: bool,
}

impl Default for SettingsState {
    fn default() -> Self {
        Self {
            active_tab: SettingsTab::General,
            providers: Vec::new(),
            loading: false,
            error_msg: String::new(),
            success_msg: String::new(),
            selected_key: PLATFORM_KEY.to_string(),
            form_name: String::new(),
            form_base_url: String::new(),
            form_api_key: String::new(),
            form_api_format: "anthropic".to_string(),
            form_models: Vec::new(),
            form_has_api_key: false,
            form_category: "custom".to_string(),
            form_preset_type: String::new(),
            form_website_url: String::new(),
            form_api_key_url: String::new(),
            form_icon: String::new(),
            form_icon_color: String::new(),
            form_sort_order: 0,
            saving: false,
            show_model_dialog: false,
            editing_model_idx: None,
            model_form_name: String::new(),
            model_form_max_tokens: "200000".to_string(),
            testing_model_idx: None,
            test_result: None,
            show_test_result: false,
        }
    }
}
