use std::env;

use serde_json::*;
use serenity::{
    async_trait,
    model::{channel::Message, gateway::Ready},
    prelude::*,
    http::client::Http,
};

struct Handler;

async fn create_support_thread(ctx: Context, name: String){
    // https://discord.com/developers/docs/resources/guild#create-guild-channel
    let value = json!({
        "name": name,
    });
    println!("xd");
    let data = ctx.data.read().await;
    match data.get::<PassedClient>().unwrap().create_private_thread(*data.get::<PassedChannelID>().unwrap(), value.as_object().unwrap()).await {
        Ok(_) => return (),
        Err(e) => {
            println!("error: {:?}", e);
        },
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let author_name = msg.author.tag();
        create_support_thread(ctx, author_name).await;
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

struct PassedChannelID;

impl TypeMapKey for PassedChannelID {
    type Value = u64;
}

struct PassedClient;

impl TypeMapKey for PassedClient {
    type Value = Http;
}

#[tokio::main]
async fn main() {
    // Configure the client with your Discord bot token in the environment.
    dotenv::dotenv().ok();
    let token = env::var("DISCORD_TOKEN").expect("Expected a Discord token in the environment");
    let support_channel_id = env::var("SUPPORT_CHANNEL_ID")
        .expect("Expected env var SUPPORT_CHANNEL_ID; this is an ID for a channel to which to attach support threads");
    let http_client = Http::new_with_token(&token);
    let mut client =
        Client::builder(&token).event_handler(Handler).await.expect("Err creating client");
    {
        let mut data = client.data.write().await;
        data.insert::<PassedClient>(http_client);
        data.insert::<PassedChannelID>(support_channel_id.parse::<u64>().unwrap());
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
