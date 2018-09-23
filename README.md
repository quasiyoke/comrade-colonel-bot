# Comrade Colonel Bot

[![Build Status](https://travis-ci.com/quasiyoke/comrade-colonel-bot.svg?branch=master)](https://travis-ci.com/quasiyoke/comrade-colonel-bot)

Telegram bot removing group chat messages after specified period of time.

To try it quickly, run:

```sh
export TELEGRAM_BOT_TOKEN=123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11
export STORAGE_PATH=/var/lib/comrade-colonel-bot/db.sqlite3
export MESSAGE_LIFETIME=5
export DELETION_PERIOD=1
cargo run
```

Don't forget to specify your actual Telegram bot token (you're able to get one from `@BotFather`). The bot will be configured to delete all messages he has evidenced after 5 seconds. [Maximal possible](https://core.telegram.org/bots/api#deletemessage) manageable chat message lifetime is 48 hours (172800 seconds).

## Deployment

1. Install Kubernetes,

1. Create Kubernetes' secret containing Telegram bot token:

   ```sh
   echo -n '123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11' > ./telegram-bot-token
   kubectl create secret generic comrade-colonel-bot --from-file=./telegram-bot-token
   ```

1. Fetch bot's deployment configuration file and run the bot:

   ```sh
   curl https://raw.githubusercontent.com/quasiyoke/comrade-colonel-bot/master/kubernetes.yml | kubectl -f -
   ```

   Default configuration assumes chat messages lifetime of 42 hours.
