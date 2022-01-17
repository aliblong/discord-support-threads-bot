# Discord Support Threads Bot

This bot serves an elegant solution for managing communication between a support team and users in a Discord server.

A **user simply direct-messages the bot with a thread title** (and a leading server ID if the user and bot share multiple servers), then the bot creates a **private thread** in that channel with the form `{requester (nick)name} | {requester-provided title}` (truncated to 100 bytes), and invites the requester to it.
The only users who can view this thread are those who have explicitly been invited to it (starting with the requester), and those with the `Manage Threads` permission, which you can restrict to the support team.
From here, anyone in the thread can invite further users.

## Motivation

A typical bot solution for support management on Discord takes the form of "mod mail", where a bot serves as an intermediary; the user's DMs to the bot are relayed to the Staff team via messages sent by the bot in a channel created specifically by that user.
This is a reasonable approach, but it has a couple of notable downsides:
1. Proliferation of channels
2. Communication feels less personal

Point 2 could be viewed as an upside, if the support team _desires_ that the user not know which member of the team is sending messages.
And the bot implementation could simply convey which team member is responding.
Nonetheless, at the end of the day, there is a certain inelegance to communicating through an intermediary.

Support via private threads is an elegant solution to both problems.
Threads can be automatically or manually archived when the issue is resolved.
Communication is directly between the user and support team, and the user can see exactly which team member is responding.
Support Threads bot simply automates the creation of such threads.

## Considerations -- should you use this bot?

There is one primary downside to this bot: **Private Threads is a premium feature on Discord**!
Access to private thread creation is currently gated behind the Server Boosts subscription model -- namely, Server Level 2, which requires 7 boosts per month.
This could cost your community as much as 35 USD/mo, or as little as nothing, if you have enough users with Discord Nitro are willing to contribute their complimentary server boosts.
