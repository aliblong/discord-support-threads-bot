# Discord Support Threads Bot

This bot serves an elegant solution for managing communication between a support team and users in a Discord server.
After the backend has been deployed with a configuration including the target server and channel, a user simply direct-messages the bot with a thread title, then the bot creates a **private thread** in that channel with the form `{requester (nick)name} | {requester-provided title}` (truncated to 100 bytes), and invites the requester to it.
Only those with the `Manage Threads` permission, which you can restrict to the support team, and those who are explicitly invited to the support thread can view it.

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

There are two main downsides to using this bot:

### Private Threads is a premium feature on Discord
Access to private thread creation is currently gated behind the Server Boosts subscription model -- namely, Server Level 2, which requires 7 boosts per month.
This could cost your community as much as 35 USD/mo, or as little as nothing, if you have enough users with Discord Nitro are willing to contribute their complimentary server boosts.

### This bot must be self-hosted
Currently, this bot doesn't support multiple servers in a single deployment, meaning I can't host it for use on your own server.
However, it's very easy to deploy yourself in any number of ways.
I'll provide a brief guide to deployment through Heroku:
1. [Create your own Discord app + bot](https://discord.com/developers/applications).
There are many guides for this.
The permissions your bot will need are `Send Messages in Threads` and `Manage Threads`.
2. Invite the bot to your server.
3. Fork this repo.
4. Create a Heroku account if you don't have one.
Add a credit card in order to unlock enough free dyno hours to run your bot 24/7 for free.
5. Create a Heroku app and connect it to your forked repo.
6. In the Resources tab of the Heroku web console, switch on the worker dyno.
7. In the Settings tab, populate `Config Vars` with the variables indicated in [`.env_template`](.env_template).
