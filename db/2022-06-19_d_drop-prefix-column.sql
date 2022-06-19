alter table guilds add column command_prefix text not null check((length(command_prefix) >= 1) and (length(command_prefix) <= 100)) default '!st ';
