use lol_rl_protocol::DEFAULT_RL_SERVER_ADDR;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                tracing_subscriber::EnvFilter::new("warn,lol_rl=info,lol_rl_protocol=info")
            }),
        )
        .init();

    println!("==========================================================================");
    println!("        LOL_RL: FIORA VS RIVEN RL TRAINING SERVER (WEBSOCKET)             ");
    println!("==========================================================================");

    let addr =
        std::env::var("RL_SERVER_ADDR").unwrap_or_else(|_| DEFAULT_RL_SERVER_ADDR.to_string());
    lol_rl::server::start_rl_server(&addr).await?;
    Ok(())
}
