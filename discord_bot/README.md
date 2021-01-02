# Advent of Code -- Discord Bot

Under construction Discord bot for the annual Advent of Code competition

Setup inspired by [this blog post](https://developers.facebook.com/blog/post/2020/09/30/build-discord-bot-with-rust-and-serenity/).

To use:

- Setup a Discord bot from: https://discord.com/developers/applications
- Get the token for the bot, this needs to be available as the environment variable `DISCORD_TOKEN`.
- Add it to a server
- Get the session cookie from https://adventofcode.com/
  This token must be available as the environment variable `AOC_SESSION`.

Get the source:

```
git clone git@github.com:jackonelli/aoc-bot.git
```

Then run:

```
cargo run --bin aoc_discord_bot
```
