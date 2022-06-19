use serenity::{
    builder::CreateApplicationCommand,
    model::interactions::application_command,
};

pub const HELP_USAGE: &'static str ="`/support_channel <channel>`";
pub const HELP_EXPLANATION: &'static str = "I will start using this channel as the location where I open support threads.";

pub async fn handle(
    ctx: serenity::client::Context,
    command: &application_command::ApplicationCommandInteraction,
) -> String {
    // recheck this assumption if we ever allow commands from DM
    let guild_id = command.guild_id.unwrap();
    let options = command
        .data
        .options
        .get(0)
        .expect("Expected channel")
        .resolved
        .as_ref().unwrap(); // Double-check if this is safe (I don't see how requiring it could result in a None option)
    let data = ctx.data.read().await;
    let pool = data.get::<crate::DbPool>().unwrap();

    // Oh yeah, it's Java naming time 8D
    if let application_command::ApplicationCommandInteractionDataOptionValue::Channel(channel) = options {
        match crate::db::update_support_channel_id(&pool, guild_id, channel.id).await {
            Ok(_) => {
                match &channel.name {
                    Some(channel_name) => format!("Successfully set support channel to #{}", &channel_name),
                    None => format!("Successfully set support channel to <#{}>", &channel.id),
                }
            }
            Err(e) => format!("Failed with {:?}", e),
        }
        
    } else {
        "Please provide a channel where I should open support threads".to_string()
    }
}

pub fn build_application_command_interface() -> CreateApplicationCommand {
    let mut command_interface = CreateApplicationCommand::default();
    command_interface
        .name("support_channel")
        .description(HELP_EXPLANATION)
        .default_member_permissions(serenity::model::permissions::Permissions::MANAGE_GUILD)
        .create_option(|option| {
            option
                .name("channel")
                .description("The channel that will host private support threads")
                .kind(application_command::ApplicationCommandOptionType::Channel)
                .required(true)
        });
    command_interface
}
