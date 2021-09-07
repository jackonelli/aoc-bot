# Advent of Code -- Discord Bot

Discord bot for the annual Advent of Code competition

Setup stolen from [this blog post](https://developers.facebook.com/blog/post/2020/09/30/build-discord-bot-with-rust-and-serenity/).

To use:

- Setup a Discord bot from: https://discord.com/developers/applications.
- Get the token, and application id for the bot.
- Add it to a server.
- Get a channel id from Discord.
- Get the session cookie from https://adventofcode.com/.
- Create a config file based on `sample_config.json`, name it `config.json`

Get the source:

```
git clone git@github.com:jackonelli/aoc-bot.git
```

Then run:

```
cargo run --bin aoc_discord_bot
```
