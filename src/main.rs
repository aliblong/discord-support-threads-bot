use std::env;
use unicode_segmentation::UnicodeSegmentation;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready, id::GuildId, id::ChannelId},
    utils::MessageBuilder,
    prelude::*,
};

struct Handler;

/// Given the requester's name and a message, create a thread title by combining them as
/// `"{requester name} | {message}`, then truncating to 100 bytes (max limit for thread name).
fn generate_thread_name(author_name: &str, msg: &str) -> String {
    const MAX_THREAD_NAME_LENGTH_BYTES: usize = 100;
    let mut byte_count = 0usize;
    author_name.graphemes(true)
        // If the argument to `graphemes` is true, the iterator is over the extended grapheme
        // clusters; otherwise, the iterator is over the legacy grapheme clusters.
        // UAX#29 recommends extended grapheme cluster boundaries for general processing.
        .chain(" | ".graphemes(true))
        .chain(msg.graphemes(true))
        .take_while(|x| {
            byte_count += x.len();
            byte_count <= MAX_THREAD_NAME_LENGTH_BYTES
        }).collect()
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.is_private() { // ignore everything but DMs
            return;
        }

        // Read guild ID and channel ID from the configuration passed to the server
        let ctx_clone = ctx.clone(); // this is required to live on its own line
        let data = ctx_clone.data.read().await;
        let bare_guild_id = *data.get::<PassedGuildID>().unwrap();
        let bare_channel_id = *data.get::<PassedChannelID>().unwrap();
        let guild_id: GuildId = (bare_guild_id).into();
        let channel_id: ChannelId = (bare_channel_id).into();

        let author = msg.author;
        // If the requester has a nickname on the guild, prefer to use that
        let author_name = match author.nick_in(ctx.clone(), guild_id).await {
            Some(nick) => nick,
            None => author.tag().split('#').next().unwrap().to_string(),
        };

        let thread_name = generate_thread_name(&author_name, &msg.content);
        match channel_id.create_private_thread(ctx.clone(), |thread_builder|
                thread_builder.name(thread_name).auto_archive_duration(4320)).await {
            Ok(thread) => {
                // could add a Staff roulette here
                thread.send_message(ctx.clone(), |msg_builder|
                    msg_builder.content(MessageBuilder::new().mention(&author)),
                ).await.unwrap();
            }
            Err(why) => {
                author.dm(ctx, |builder|
                    builder.content(format!("Your thread could not be created: {:?}", why))
                ).await.unwrap();
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

// This way of passing data to the bot strikes me as odd.
// Basically, the data is massaged into a TypeMap, which as the name implies,
// maps _types_ to arbitrary data.
// I suppose this is a way of enforcing usage of newtypes.
/// A wrapper of sorts over a u64 value which will be used as a Guild ID.
/// The wrapper is for use with [`serenity::prelude::TypeMap`], which is the data format
/// used to pass configuration data to the bot.
struct PassedGuildID;
/// A wrapper of sorts over a u64 value which will be used as a Channel ID.
/// The wrapper is for use with [`serenity::prelude::TypeMap`], which is the data format
/// used to pass configuration data to the bot.
struct PassedChannelID;

impl TypeMapKey for PassedGuildID {
    type Value = u64;
}

impl TypeMapKey for PassedChannelID {
    type Value = u64;
}

#[tokio::main]
async fn main() {
    // Simply include your configuration in a `.env` file
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a Discord token in the environment");
    let guild_id = env::var("GUILD_ID")
        .expect("Expected env var GUILD_ID; \
            this is the ID for the guild (server)  where this bot will be operating");
    let support_channel_id = env::var("SUPPORT_CHANNEL_ID")
        .expect("Expected env var SUPPORT_CHANNEL_ID; \
            this is an ID for a channel to which to attach support threads");
    let mut client =
        Client::builder(&token).event_handler(Handler).await.expect("Err creating client");
    {
        // Insert configuration into the bot
        let mut data = client.data.write().await;
        data.insert::<PassedGuildID>(guild_id.parse::<u64>().unwrap());
        data.insert::<PassedChannelID>(support_channel_id.parse::<u64>().unwrap());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
