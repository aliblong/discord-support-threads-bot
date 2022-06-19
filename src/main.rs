use anyhow::Result;
use clap::{Parser, Subcommand};
use std::env::var;

use serenity::{
    client::Context,
    model::{gateway, interactions::Interaction},
    prelude::TypeMapKey,
};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    subcommand: Subcommands,
}

#[derive(Subcommand, PartialEq)]
enum Subcommands {
    Bot,
    RegisterSlashCommands,
    UnregisterSlashCommands,
}

pub type HttpClient = serenity::http::client::Http;

mod commands;
mod db;

struct Handler;

#[serenity::async_trait]
impl serenity::client::EventHandler for Handler {
    async fn interaction_create(&self, ctx: Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            commands::handle_application_command(ctx, command).await;
        }
    }

    async fn ready(&self, ctx: Context, ready: gateway::Ready) {
        ctx.set_activity(gateway::Activity::listening("/help"))
            .await;
        println!("{} is connected!", ready.user.name);
    }
}
pub struct DbPool;

impl TypeMapKey for DbPool {
    type Value = sqlx::postgres::PgPool;
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    env_logger::init();
    // Simply include your configuration in a `.env` file
    dotenv::dotenv().ok();
    let token = var("DISCORD_TOKEN").expect("Expected a Discord token in the environment");
    let bot_user_id = var("BOT_USER_ID")
        .expect("Expected env var BOT_USER_ID")
        .parse::<u64>()?;
    let optional_target_guild_id = match var("TARGET_GUILD_ID") {
        Ok(guild_id_str) => Some(guild_id_str.parse::<u64>()?.into()),
        Err(_) => None,
    };
    match &cli.subcommand {
        Subcommands::Bot => {
            let database_url = var("DATABASE_URL").expect("Expected env var DATABASE_URL");
            // It doesn't feel like it matters whether we use `connect` or `connect_lazy`
            let db_pool = sqlx::postgres::PgPoolOptions::new()
                .connect(&database_url)
                .await?;
            let framework = serenity::framework::standard::StandardFramework::new();
            let mut client =
                serenity::client::Client::builder(&token, gateway::GatewayIntents::empty())
                    .event_handler(Handler)
                    .application_id(bot_user_id)
                    .framework(framework)
                    .await
                    .expect("Err creating client");
            {
                // Insert configuration into the bot
                let mut data = client.data.write().await;
                data.insert::<DbPool>(db_pool);
            }

            if let Err(why) = client.start().await {
                log::error!("Client error: {:?}", why);
            }
        }
        Subcommands::RegisterSlashCommands | Subcommands::UnregisterSlashCommands => {
            let client = HttpClient::new_with_application_id(&token, bot_user_id);
            if let Some(target_guild_id) = optional_target_guild_id {
                commands::unregister_all_guild_application_commands(&client, target_guild_id).await;
                if cli.subcommand == Subcommands::RegisterSlashCommands {
                    commands::register_guild_application_commands(&client, target_guild_id).await;
                }
            } else {
                commands::unregister_all_global_application_commands(&client).await;
                if cli.subcommand == Subcommands::RegisterSlashCommands {
                    commands::register_global_application_commands(&client).await;
                }
            }
        }
    }
    Ok(())
}
