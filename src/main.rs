use std::env;
use unicode_segmentation::UnicodeSegmentation;

use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
};

struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        if !msg.is_private() { // ignore everything but DMs
            return;
        }
        let ctx_clone = ctx.clone(); // this is required to live on its own line
        let data = ctx_clone.data.read().await;
        let bare_guild_id = *data.get::<PassedGuildID>().unwrap();
        let bare_channel_id = *data.get::<PassedChannelID>().unwrap();
        let guild_id: serenity::model::id::GuildId = (bare_guild_id).into();
        let channel_id: serenity::model::id::ChannelId = (bare_channel_id).into();
        let author = msg.author;
        let author_name = match author.nick_in(ctx.clone(), guild_id).await {
            Some(nick) => nick,
            None => author.tag().split('#').next().unwrap().to_string(),
        };
        const MAX_THREAD_NAME_LENGTH_BYTES: usize = 100;
        let mut byte_count = 0usize;
        let thread_name: String = author_name.graphemes(true)
            // If the argument to `graphemes` is true, the iterator is over the extended grapheme
            // clusters; otherwise, the iterator is over the legacy grapheme clusters.
            // UAX#29 recommends extended grapheme cluster boundaries for general processing.
            .chain(": ".graphemes(true))
            .chain(msg.content.graphemes(true))
            .take_while(|x| {
                byte_count += x.len();
                byte_count <= MAX_THREAD_NAME_LENGTH_BYTES
            }).collect();
        channel_id.create_private_thread(ctx, |c| c.name(thread_name)).await;

    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct PassedGuildID;
struct PassedChannelID;

impl TypeMapKey for PassedGuildID {
    type Value = u64;
}

impl TypeMapKey for PassedChannelID {
    type Value = u64;
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a Discord token in the environment");
    let guild_id = env::var("GUILD_ID")
        .expect("Expected env var GUILD_ID; this is the ID for the guild (server) where this bot will be operating");
    let support_channel_id = env::var("SUPPORT_CHANNEL_ID")
        .expect("Expected env var SUPPORT_CHANNEL_ID; this is an ID for a channel to which to attach support threads");
    let mut client =
        Client::builder(&token).event_handler(Handler).await.expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<PassedGuildID>(guild_id.parse::<u64>().unwrap());
        data.insert::<PassedChannelID>(support_channel_id.parse::<u64>().unwrap());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
