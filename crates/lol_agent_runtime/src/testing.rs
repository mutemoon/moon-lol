use rig::client::CompletionClient;
use rig::completion::Prompt;
use rig::providers::anthropic;

/// 测试大模型连接：通过 rig 构建 agent 并发送一条测试消息，成功时返回大模型的回复。
pub async fn test_model_connection(
    api_key: &str,
    base_url: &str,
    model: &str,
    max_tokens: Option<u32>,
) -> Result<String, String> {
    let client = anthropic::Client::builder()
        .api_key(api_key)
        .base_url(base_url)
        .build()
        .map_err(|e| format!("构建 rig 客户端失败: {}", e))?;

    let limit = max_tokens.unwrap_or(1024);

    let agent = client
        .agent(model)
        .max_tokens(limit as u64)
        .preamble("You are a connection test assistant. Please reply with a short message (e.g. 'Hello! Connection successful.') to confirm you are online.")
        .build();

    agent
        .prompt("Hello, this is a connection test request. Are you online? Please reply briefly.")
        .await
        .map_err(|e| format!("连接测试失败: {}", e))
}
