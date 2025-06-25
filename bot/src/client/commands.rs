use super::Context;
#[poise::command(slash_command)]
pub async fn map(ctx: Context<'_>) -> anyhow::Result<()> {
    ctx.say("pong").await?;
    Ok(())
}