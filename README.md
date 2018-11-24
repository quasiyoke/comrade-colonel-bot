# Comrade Colonel Bot

[![Build Status](https://travis-ci.com/quasiyoke/comrade-colonel-bot.svg?branch=master)](https://travis-ci.com/quasiyoke/comrade-colonel-bot)

Telegram bot removing group chat messages after specified period of time.

To try it quickly, run:

```sh
export telegram_bot_token=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
export STORAGE_PATH=/var/lib/comrade-colonel-bot/db.sqlite3
export MESSAGE_LIFETIME=5
export DELETION_PERIOD=1
cargo run
```

Don't forget to specify your actual Telegram bot token (you're able to get one from `@BotFather`). The bot will be configured to delete all messages he has evidenced after 5 seconds. [Maximal possible](https://core.telegram.org/bots/api#deletemessage) manageable chat message lifetime is 48 hours (172800 seconds).

## Deployment

1. Install [Docker Compose](https://docs.docker.com/compose/install/),

1. Create a file with Telegram bot token to put it into Docker Secrets' store:

   ```sh
   echo -n '123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11' > ./telegram-bot-token
   ```

1. Fetch bot's deployment configuration file and run the bot:

   ```sh
   wget https://raw.githubusercontent.com/quasiyoke/comrade-colonel-bot/master/docker-compose.yml
   docker-compose up
   ```

   Default configuration assumes chat messages lifetime of 42 hours.
