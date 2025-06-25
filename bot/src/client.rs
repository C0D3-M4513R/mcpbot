mod commands;

use poise::serenity_prelude as serenity;
use serenity::utils::validate_token;

/// User data, which is stored and accessible in all command invocations
struct Data;
type Context<'a> = poise::Context<'a, Data, anyhow::Error>;
pub async fn init_client() -> ::anyhow::Result<serenity::Client> {
    tracing::info!("Client Startup");

    tracing::debug!("Getting Client Token");
    let token = std::env::var("DISCORD_TOKEN")?;
    validate_token(&token)?;
    
    let framework = poise::framework::FrameworkBuilder::default()
        .options(poise::FrameworkOptions {
            commands: vec![
                commands::map(),
            ],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data)
            })
        })
        .initialize_owners(true)
        .build();

    let mut client = serenity::Client::builder(&token, Default::default())
        .framework(framework)
        .await?;

    client.start().await?;

    Ok(client)
}