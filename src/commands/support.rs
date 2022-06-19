use anyhow::Result;
use const_format::concatcp;
use serenity::{
    builder::CreateApplicationCommand,
    client::Context,
    model::{id::GuildId, interactions::application_command},
};
use unicode_segmentation::{Graphemes, UnicodeSegmentation};

pub const HELP_USAGE: &'static str = "`/support <thread-title>`";
// This needs to be <= 100 characters, according to Discord API requirements:
// https://discord.com/developers/docs/interactions/application-commands#application-command-object-application-command-structure
pub const HELP_EXPLANATION: &'static str =
    "I will create a support thread with your supplied title.";
pub const HELP_EXPLANATION_EXTENDED: &'static str = concatcp!(HELP_EXPLANATION, "\n\t\t(appended to your nickname here, i.e. `your-nick | thread-title`, and truncated to 100 characters)");

#[derive(thiserror::Error, Debug)]
pub enum SupportThreadError {
    #[error(
        "This server has not been configured with /support_channel. \
            Please advise an admin there to run this command."
    )]
    UnconfiguredGuild,
}

pub async fn handle(
    ctx: Context,
    command: &application_command::ApplicationCommandInteraction,
) -> String {
    // recheck this assumption if we ever allow commands from DM
    let guild_id = command.guild_id.unwrap();
    let requester = &command.user;
    let options = command
        .data
        .options
        .get(0)
        .expect("Expected thread title")
        .resolved
        .as_ref()
        .unwrap(); // Double-check if this is safe (I don't see how requiring it could result in a None option)

    // Oh yeah, it's Java naming time 8D
    if let application_command::ApplicationCommandInteractionDataOptionValue::String(title) =
        options
    {
        let title_graphemes = title.graphemes(true);
        let requester_name = match requester.nick_in(ctx.clone(), guild_id).await {
            Some(nick) => nick,
            None => requester.tag().split('#').next().unwrap().to_string(),
        };
        let processed_title = generate_thread_name(&requester_name, title_graphemes);
        match fetch_support_channel_id_and_create_thread(
            ctx.clone(),
            guild_id,
            requester,
            processed_title,
        )
        .await
        {
            Ok(_) => "Created your support thread!".to_string(),
            Err(e) => format!("Failed with {:?}", e),
        }
    } else {
        "Please provide a title for the support thread".to_string()
    }
}

pub fn build_application_command_interface() -> CreateApplicationCommand {
    let mut command_interface = CreateApplicationCommand::default();
    command_interface
        .name("support")
        .description(HELP_EXPLANATION)
        // permissions required to execute the command
        // seems like requiring Permissions::empty() doesn't actually work
        //.default_member_permissions(serenity::model::permissions::Permissions::VIEW_CHANNEL)
        .create_option(|option| {
            option
                // Putting a space in the arg to `name` will cause the whole `set_application_commands`
                // function to silently fail!
                .name("title")
                .description("The title for your support thread")
                .kind(application_command::ApplicationCommandOptionType::String)
                .required(true)
        });
    command_interface
}

/// Given the requester's name and a message, create a thread title by combining them as
/// `"{requester name} | {message}`, then truncating to 100 bytes (max limit for thread name).
fn generate_thread_name(author_name: &str, msg_graphemes: Graphemes) -> String {
    const MAX_THREAD_NAME_LENGTH_BYTES: usize = 100;
    let mut byte_count = 0usize;
    author_name
        .graphemes(true)
        // If the argument to `graphemes` is true, the iterator is over the extended grapheme
        // clusters; otherwise, the iterator is over the legacy grapheme clusters.
        // UAX#29 recommends extended grapheme cluster boundaries for general processing.
        .chain(" | ".graphemes(true))
        .chain(msg_graphemes)
        .take_while(|x| {
            byte_count += x.len();
            byte_count <= MAX_THREAD_NAME_LENGTH_BYTES
        })
        .collect()
}

async fn fetch_support_channel_id_and_create_thread(
    ctx: Context,
    guild_id: GuildId,
    requester: &serenity::model::user::User,
    thread_name: String,
) -> Result<()> {
    let data = ctx.data.read().await;
    let pool = data.get::<crate::DbPool>().unwrap();
    match crate::db::get_support_channel_id(pool, guild_id).await? {
        None => Err(SupportThreadError::UnconfiguredGuild.into()),
        Some(support_channel_id) => {
            let thread = support_channel_id
                .create_private_thread(ctx.clone(), |thread_builder| {
                    thread_builder.name(thread_name).auto_archive_duration(4320)
                })
                .await?;
            thread
                .send_message(ctx.clone(), |msg_builder| {
                    msg_builder.content(serenity::utils::MessageBuilder::new().mention(requester))
                })
                .await?;
            Ok(())
        }
    }
}
