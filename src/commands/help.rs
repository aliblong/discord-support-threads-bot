use const_format::{concatcp, formatcp};
use serenity::builder::CreateApplicationCommand;

use crate::commands::{support, support_channel};

const HELP_MESSAGE: &'static str = formatcp!(
    "\
Hi! My job is to connect you with the administrators of this server!
I can create a _support thread_ that is _private_: only you and the admins can see it it.
Y'all can explicitly invite someone else to the thread by pinging them there.

I support one command for everyone, and one command for administrators

**Everyone commands**:
{everyone_commands}

**Admin commands**:
{admin_commands}
",
    everyone_commands = EVERYONE_COMMANDS_DOCS,
    admin_commands = ADMIN_COMMANDS_DOCS,
);
const EVERYONE_COMMANDS_DOCS: &'static str = concatcp!(
    '\t',
    support::HELP_USAGE,
    '\t',
    support::HELP_EXPLANATION_EXTENDED,
);
const ADMIN_COMMANDS_DOCS: &'static str = concatcp!(
    '\t',
    support_channel::HELP_USAGE,
    '\t',
    support_channel::HELP_EXPLANATION,
);

pub fn handle() -> String {
    HELP_MESSAGE.to_string()
}

pub fn build_application_command_interface() -> CreateApplicationCommand {
    let mut command_interface = CreateApplicationCommand::default();
    command_interface
        .name("help")
        .description("Learn how to use this bot")
        // seems like requiring Permissions::empty() doesn't actually work
        //.default_member_permissions(serenity::model::permissions::Permissions::empty())
        ;
    command_interface
}
