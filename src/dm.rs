use unicode_segmentation::{UnicodeSegmentation, Graphemes};
use anyhow::Result;

use serenity::{
    model::{
        channel::Message,
        id::{GuildId, ChannelId}
    },
    utils::MessageBuilder,
    prelude::Context,
};

#[derive(thiserror::Error, Debug)]
pub enum DmError {
    #[error("None of your common guilds has been configured. \
            Please advise someone with the Manage Server permission there to run the set_support_channel command.")]
    NoConfiguredGuilds,
    #[error("Guild with ID {guild_id} has not been configured. \
            Please advise someone with the Manage Server permission there to run the set_support_channel command.")]
    UnconfiguredGuild { guild_id: GuildId },
    #[error("Guild with ID {guild_id} either does not use this bot, or is not one of your guilds. \
        Please double check that this Guild ID is correct.")]
    WrongGuildId { guild_id: GuildId },
    #[error("Since you are in multiple servers that use me, you must write as the first part \
        of your message the ID of the server in which you would like to open a support thread.\n\
        If you don't know how to find this ID, please consult this support article: \
        https://support.discord.com/hc/en-us/articles/206346498-Where-can-I-find-my-User-Server-Message-ID-\n\
        Usage example with this bot: `91238421834712347 my thread title`\n\
        Here are the IDs of servers you share with me that have been properly configured for its use:\n\
        {formatted_configured_guild_ids}")]
    UnspecifiedGuildId { formatted_configured_guild_ids: String },
    #[error("The Guild ID you supplied contained a wrong character: `{wrong_char}`")]
    WrongCharInGuildId { wrong_char: char },
}

/// Given the requester's name and a message, create a thread title by combining them as
/// `"{requester name} | {message}`, then truncating to 100 bytes (max limit for thread name).
pub fn generate_thread_name(author_name: &str, msg_graphemes: Graphemes) -> String {
    const MAX_THREAD_NAME_LENGTH_BYTES: usize = 100;
    let mut byte_count = 0usize;
    author_name.graphemes(true)
        // If the argument to `graphemes` is true, the iterator is over the extended grapheme
        // clusters; otherwise, the iterator is over the legacy grapheme clusters.
        // UAX#29 recommends extended grapheme cluster boundaries for general processing.
        .chain(" | ".graphemes(true))
        .chain(msg_graphemes)
        .take_while(|x| {
            byte_count += x.len();
            byte_count <= MAX_THREAD_NAME_LENGTH_BYTES
        }).collect()
}

    /*
    // seems like streams, i.e. async iterators, are not stable yet, so gotta process guild lists
    // the old-fashioned way
    
       .filter(|guild_id_and_support_channel_id| {
           // idk if it's possible to destructure the tuple more conveniently than this
           let guild_id = guild_id_and_support_channel_id.0;
           // this function feels like it should return a Result<Option<User>>
           // but maybe I'm missing some easier way to test membership in a guild?
           guild_id.member(ctx, author).await.is_ok()
       }).collect();
    let mutual_guilds_that_are_unconfigured = ordered_mutual_guilds.drain_filter(
        |guild_id_and_support_channel_id| {
            let support_channel_id = guild_id_and_support_channel_id.0;
            support_channel_id.is_none()
        });
    */

/// If the bot and user have only one common guild, it's that one.
/// If they have several common guilds, then their message needs to begin with the ID of the guild
/// they wish to target.
async fn discern_guild<'a>(ctx: &Context, msg: &'a Message) -> Result<(GuildId, ChannelId, Graphemes<'a>)> {
    // This pattern appears several times, but I don't know how to factor it out
    let data = ctx.data.read().await;
    let pool = data.get::<crate::DbPool>().unwrap();

    let author = &msg.author;
    let mut ordered_mutual_guilds_configured: Vec<(GuildId, ChannelId)> = Vec::new();
    let mut ordered_mutual_guilds_unconfigured: Vec<GuildId> = Vec::new();
    let mut msg_graphemes = msg.content.graphemes(true);
    // Seems to be no way to check mutual guilds,
    // so have to check user's membership in each one in the db.
    for (guild_id, optional_channel_id) in
            crate::db::get_list_of_supported_guilds_and_channels_ordered_by_guild_id(pool).await?.into_iter() {
        // This function is a simple way to check guild membership, so it should probably return
        // some sort of Option rather than corresponding no membership to an Err val.
        // I'm throwing away the error information, so if there is a genuine error, it will be
        // treated as if the user simply isn't in the guild.
        if guild_id.member(&ctx, author).await.is_ok() {
            match optional_channel_id {
                Some(support_channel_id) => ordered_mutual_guilds_configured
                    .push((guild_id, support_channel_id)),
                None => ordered_mutual_guilds_unconfigured.push(guild_id)
            }
        }
    }

    match ordered_mutual_guilds_configured.len() {
        0 => {
            Err(DmError::NoConfiguredGuilds.into())
        }
        1 => {
            let (guild_id, channel_id) = ordered_mutual_guilds_configured[0];
            return Ok((guild_id, channel_id, msg_graphemes));
        }
        _ => {
            let target_guild = parse_guild_id_from_message(&mut msg_graphemes, &ordered_mutual_guilds_configured)?;
            match ordered_mutual_guilds_configured.binary_search_by(
                |guild_id_and_support_channel_id| {
                    guild_id_and_support_channel_id.0.cmp(&target_guild)
                }
            ) {
                Ok(idx) => {
                    let (guild_id, channel_id) = ordered_mutual_guilds_configured[idx];
                    Ok((guild_id, channel_id, msg_graphemes))
                }
                Err(_) => {
                    match ordered_mutual_guilds_unconfigured.binary_search(&target_guild) {
                        Ok(_) => Err(
                            DmError::UnconfiguredGuild{guild_id: target_guild}.into()),
                        Err(_) => Err(
                            DmError::WrongGuildId{guild_id: target_guild}.into()),
                    }
                }
            }
        }
    }
}

/// This is done with graphemes rather than chars simply because we desire to use graphemes at a
/// later stage.
fn parse_guild_id_from_message(
    msg_graphemes: &mut Graphemes,
    ordered_mutual_guilds_configured: &Vec<(GuildId, ChannelId)>,
) -> Result<GuildId> {
    let mut guild_id_builder = Vec::<char>::new();
    while let Some(g) = msg_graphemes.next() {
        match g.len() {
            0 => unreachable!(),
            1 => {
                let c = g.chars().next().unwrap();
                if c.is_ascii_digit() {
                    guild_id_builder.push(c)
                } else {
                    if guild_id_builder.len() == 0 {
                        return Err(DmError::UnspecifiedGuildId{
                            formatted_configured_guild_ids: 
                                ordered_mutual_guilds_configured.iter().map(|guild_id_and_support_channel_id| {
                                    guild_id_and_support_channel_id.0.0.to_string()
                                }).collect::<Vec<String>>().join("\n")
                        }.into());
                    }

                    if c.is_whitespace() {
                        break;
                    } else {
                        return Err(DmError::WrongCharInGuildId{ wrong_char: c }.into());
                    }
                }
            }
            _ => break,
        }
    }
    // the only way this parse can fail is if the number is too large to be held in a u64
    let target_guild: GuildId =
        guild_id_builder.into_iter().collect::<String>().parse::<u64>()?.into();
    Ok(target_guild)
}

pub async fn discern_guild_then_create_support_thread(ctx: &Context, msg: &Message) -> Result<()> {
    let (guild_id, support_channel_id, msg_remainder_graphemes) = discern_guild(ctx, msg).await?;

    // If the requester has a nickname on the guild, prefer to use that
    let author = &msg.author;
    let author_name = match author.nick_in(ctx.clone(), guild_id).await {
        Some(nick) => nick,
        None => author.tag().split('#').next().unwrap().to_string(),
    };

    let thread_name = generate_thread_name(&author_name, msg_remainder_graphemes);
    let thread = support_channel_id.create_private_thread(
        ctx.clone(),
        |thread_builder| thread_builder.name(thread_name).auto_archive_duration(4320)
    ).await?;
    // could add a Staff roulette here
    thread.send_message(ctx.clone(), |msg_builder|
        msg_builder.content(MessageBuilder::new().mention(author)),
    ).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::dm::{parse_guild_id_from_message, DmError};
    use unicode_segmentation::{UnicodeSegmentation, Graphemes};
    use serenity::model::id::{GuildId, ChannelId};
    #[test]
    fn guild_id_parsing_well_formed() {
        let msg = "1234 test".to_string();
        let mut msg_graphemes = msg.graphemes(true);
        let guild_id = parse_guild_id_from_message(&mut msg_graphemes, &vec![(1234u64.into(), 5678u64.into())]);
        assert!(guild_id.is_ok());
        let target: GuildId = 1234u64.into();
        assert_eq!(guild_id.unwrap(), target);
    }
    #[test]
    fn guild_id_parsing_bad_char() {
        let msg = "1234a test".to_string();
        let mut msg_graphemes = msg.graphemes(true);
        let guild_id = parse_guild_id_from_message(&mut msg_graphemes, &vec![(1234u64.into(), 5678u64.into())]);
        assert!(guild_id.is_err());
        // `anyhow` makes it feel rather pointless to test further than just `is_err`
        //assert_eq!(guild_id.unwrap_err(), DmError::WrongCharInGuildId{wrong_char: 'a'});
    }
    #[test]
    fn guild_id_parsing_no_guild_id() {
        let msg = "a test".to_string();
        let mut msg_graphemes = msg.graphemes(true);
        let guild_id = parse_guild_id_from_message(&mut msg_graphemes, &vec![(1234u64.into(), 5678u64.into())]);
        assert!(guild_id.is_err());
        // `anyhow` makes it feel rather pointless to test further than just `is_err`
        //assert_eq!(guild_id.unwrap_err(), DmError::WrongCharInGuildId{wrong_char: 'a'});
    }
}
