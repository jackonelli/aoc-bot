# Advent of Code -- Discord Bot

Under construction Discord bot for the annual Advent of Code competition

Setup inspired by [this blog post](https://developers.facebook.com/blog/post/2020/09/30/build-discord-bot-with-rust-and-serenity/).

To use:

- Setup a Discord bot from: https://discord.com/developers/applications.
- Get the token, and application id for the bot.
- Add it to a server.
- Get the session cookie from https://adventofcode.com/.
- Get a channel id from Discord.
- Create a config file based on `sample_config.json`

Get the source:

```
git clone git@github.com:jackonelli/aoc-bot.git
```

Then run:

```
cargo run --bin aoc_discord_bot
```
