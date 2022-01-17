use serenity::prelude::*;

#[serenity::framework::standard::macros::command]
#[aliases("set-command-prefix", "set_prefix", "set-prefix", "prefix")]
#[only_in("guild")]
#[required_permissions(MANAGE_GUILD)]
#[num_args(1)]
async fn set_command_prefix(
    ctx: &Context,
    msg: &serenity::model::channel::Message,
    mut args: serenity::framework::standard::Args
) -> serenity::framework::standard::CommandResult {
    // Takes the first (and only) argument, trimmed and de-quoted
    // I believe the num_args attribute macro guarantees that this won't panic
    let prefix = args.trimmed().single_quoted::<String>().unwrap(); 

    // This pattern appears several times, but I don't know how to factor it out
    let data = ctx.data.read().await;
    let pool = data.get::<crate::DbPool>().unwrap();

    // Message should always have an associated guild ID, because this handler is configured only
    // to respond to messages sent within a guild.
    match crate::db::update_prefix(pool, &msg.guild_id.unwrap(), &prefix).await {
        Ok(_) => msg.reply(ctx, format!("Updated my command prefix to {}", prefix)).await,
        Err(why) => msg.reply(ctx, format!("Failed to update my command prefix. {:?}", why)).await,
    };
    Ok(())
}
