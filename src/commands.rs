use crate::HttpClient;
use serenity::{
    builder::CreateApplicationCommands,
    model::{
        id::GuildId,
        interactions,
    },
};

pub mod help;
pub mod support;
pub mod support_channel;

pub async fn handle_application_command(
    ctx: serenity::client::Context,
    command: interactions::application_command::ApplicationCommandInteraction,
) {
    let reply_content = match command.data.name.as_str() {
        "support" => support::handle(ctx.clone(), &command).await,
        "support_channel" => support_channel::handle(ctx.clone(), &command).await,
        "help" => help::handle(),
        _ => "command not implemented".to_string(),
    };

    match command
        .create_interaction_response(&ctx.http, |response| {
            response
                .kind(interactions::InteractionResponseType::ChannelMessageWithSource)
                .interaction_response_data(|message| {
                    message.content(reply_content).flags(
                        interactions::InteractionApplicationCommandCallbackDataFlags::EPHEMERAL,
                    )
                })
        })
        .await
    {
        Ok(()) => (),
        Err(err) => log::error!("{:?}", err),
    }
}

pub async fn unregister_all_global_application_commands(client: &HttpClient) {
    log::info!("Unregistering all global application commands");
    for ac in client.get_global_application_commands().await.unwrap() {
        log::info!("{:?}", ac);
        client
            .delete_global_application_command(ac.id.into())
            .await
            .unwrap();
    }
}

pub async fn unregister_all_guild_application_commands(
    client: &HttpClient,
    target_guild_id: GuildId,
) {
    log::info!(
        "Unregistering all application commands for guild {}",
        target_guild_id
    );
    // The http client API seems to be designed more for use by the bot client than for use as a library.
    // Taking raw types as args (e.g. u64 rather than GuildId, json::Value instead of builder fn)
    let guild_id: u64 = target_guild_id.into();
    for ac in client
        .get_guild_application_commands(guild_id)
        .await
        .unwrap()
    {
        log::info!("{:?}", ac);
        client
            .delete_guild_application_command(guild_id, ac.id.into())
            .await
            .unwrap();
    }
}

pub async fn register_global_application_commands(client: &HttpClient) {
    let commands = build_application_commands_interfaces();
    // copied invocation from
    // https://docs.rs/serenity/latest/src/serenity/model/guild/guild_id.rs.html#1456-1469
    match client
        .create_global_application_commands(&serenity::json::Value::from(commands.0))
        .await
    {
        Ok(_) => (),
        Err(err) => {
            log::error!("Error creating global application commands");
            log::error!("{:?}", err);
        }
    };
}

fn build_application_commands_interfaces() -> CreateApplicationCommands {
    let mut commands = CreateApplicationCommands::default();
    commands
        .add_application_command(help::build_application_command_interface())
        .add_application_command(support::build_application_command_interface())
        .add_application_command(support_channel::build_application_command_interface());
    commands
}

pub async fn register_guild_application_commands(client: &HttpClient, target_guild_id: GuildId) {
    let commands = build_application_commands_interfaces();
    // copied invocation from
    // https://docs.rs/serenity/latest/src/serenity/model/guild/guild_id.rs.html#1456-1469
    match client
        .create_guild_application_commands(
            target_guild_id.into(),
            &serenity::json::Value::from(commands.0),
        )
        .await
    {
        Ok(_) => (),
        Err(err) => {
            log::error!(
                "Error creating application commands for guild {}",
                target_guild_id
            );
            log::error!("{:?}", err);
        }
    };
}
