create table if not exists guilds (
    id bigint primary key
  , support_channel_id bigint
  , command_prefix text not null check((length(command_prefix) >= 1) and (length(command_prefix) <= 100)) default '!st '
);
