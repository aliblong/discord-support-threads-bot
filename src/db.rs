use sqlx::postgres::PgPool;
use serenity::model::id::{GuildId, ChannelId};
use anyhow::Result;


pub async fn get_list_of_configured_guilds(db_pool: &PgPool)
        -> Result<Vec<GuildId>> {
    let guilds = sqlx::query!(
        "select id
        from guilds
        where support_channel_id is not null"
    ).fetch_all(db_pool).await?
        .into_iter().map(|record| {
            (record.id as u64).into() // GuildId
        }).collect();
    Ok(guilds)
}

pub async fn get_list_of_supported_guilds_and_channels_ordered_by_guild_id(db_pool: &PgPool)
        -> Result<Vec<(GuildId, Option<ChannelId>)>> {
    let guilds = sqlx::query!(
        "select id, support_channel_id
        from guilds
        order by id"
    ).fetch_all(db_pool).await?
        .into_iter().map(|record| {
            (
                (record.id as u64).into(), // GuildId
                (record.support_channel_id.map(|id| (id as u64).into())), // Option<ChannelId>
            )
        }).collect();
    Ok(guilds)
}

pub async fn update_prefix(db_pool: &PgPool, guild_id: &GuildId, prefix: &str) -> Result<()> {
    let raw_guild_id = guild_id.0 as i64;
    let support_channel_id = sqlx::query!(
        "update guilds
        set command_prefix = $2
        where id = $1",
        raw_guild_id,
        prefix,
    ).execute(db_pool).await?;
    Ok(())
}

pub async fn update_support_channel_id(db_pool: &PgPool, guild_id: GuildId, support_channel_id: ChannelId) -> Result<()> {
    let raw_guild_id = guild_id.0 as i64;
    let support_channel_id = sqlx::query!(
        "update guilds
        set support_channel_id = $2
        where id = $1",
        raw_guild_id,
        support_channel_id.0 as i64,
    ).execute(db_pool).await?;
    Ok(())
}

pub async fn get_support_channel_id(db_pool: &PgPool, guild_id: GuildId) -> Result<Option<ChannelId>> {
    let raw_guild_id = guild_id.0 as i64;
    let support_channel_id = sqlx::query!(
        "select support_channel_id
        from guilds
        where id = $1",
        raw_guild_id
    ).fetch_one(db_pool).await?.support_channel_id.map(|id| (id as u64).into());
    Ok(support_channel_id)
}
