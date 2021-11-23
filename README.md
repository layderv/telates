(Unmaintained) Telegram bot to send messages when subscribed feed rss's are updated. Some bugs here and there.

It uses teloxide and redis. Telegram users are allowed through a .txt file with their IDs. env.list must contain the environment variables to use the Telegram API

* docker build -t telates .
* docker run --env-file ./env.list -d -t telates
