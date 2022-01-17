use anyhow::anyhow;
use serenity::prelude::*;

#[serenity::framework::standard::macros::command]
#[aliases("set-support-channel", "support-channel", "supchan")]
#[only_in("guild")]
#[required_permissions(MANAGE_GUILD)]
#[num_args(1)]
async fn set_support_channel(
    ctx: &Context,
    msg: &serenity::model::channel::Message,
    mut args: serenity::framework::standard::Args
) -> serenity::framework::standard::CommandResult {
    let channel_id;

    // unfortunately msg.mention_channels does not provide what we're looking for
    // see its documentation for more details
    let arg = args.trimmed().single_quoted::<String>().unwrap();
    match arg.parse::<serenity::model::id::ChannelId>() {
        Ok(parsed_channel_id) => channel_id = parsed_channel_id,
        Err(_) => {
            let err_msg = "You must provide precisely one channel to the set_support_channel command";
            msg.reply(ctx, err_msg).await?;
            return Err(anyhow!(err_msg).into());
        }
    }

    // This pattern appears several times, but I don't know how to factor it out
    let data = ctx.data.read().await;
    let pool = data.get::<crate::DbPool>().unwrap();

    // Message should always have an associated guild ID, because this handler is configured only
    // to respond to messages sent within a guild.
    crate::db::update_support_channel_id(pool, msg.guild_id.unwrap(), channel_id.0 as i64).await?;
    msg.reply(ctx, format!("Support channel has been updated to {}", arg)).await?;
    Ok(())
}
