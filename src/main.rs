#[macro_use]
extern crate derive_more;

mod states;
mod transitions;
mod refresher;
mod client;

use states::*;

use teloxide::{
    dispatching::dialogue::{serializer::Bincode, RedisStorage, Storage},
    prelude::*,
};
use thiserror::Error;
use teloxide::types::User;
use std::fs::File;
use std::io::{BufReader, BufRead};
use crate::refresher::Refresher;
use std::thread;

type StorageError = <RedisStorage<Bincode> as Storage<Dialogue>>::Error;

#[derive(Debug, Error)]
enum Error {
    #[error("error from Telegram: {0}")]
    TelegramError(#[from] RequestError),
    #[error("error from storage: {0}")]
    StorageError(#[from] StorageError),
}

type In = DialogueWithCx<Message, Dialogue, StorageError>;

#[tokio::main]
async fn main() {
    teloxide::enable_logging!();
    run().await;
}

async fn run() {
    let bot = Bot::from_env();
    let bg_bot = bot.clone();
    thread::spawn(move || {
        Refresher::new(bg_bot).run()
    });
    Dispatcher::new(bot)
        .messages_handler(DialogueDispatcher::with_storage(
            |DialogueWithCx { cx, dialogue }: In| async move {
                let dialogue = dialogue.expect("std::convert::Infallible");
                handle_message(cx, dialogue).await.expect("Something wrong with the bot!")
            },
            RedisStorage::open("redis://127.0.0.1:6379", Bincode).await.unwrap(),
        ))
        .dispatch()
        .await;
}

async fn handle_message(cx: UpdateWithCx<Message>, dialogue: Dialogue) -> TransitionOut<Dialogue> {
    match cx.update.from() {
        Some(user) if user_allowed(&user) => {
            log::info!("started on chat_id: {}", cx.chat_id());
            match cx.update.text_owned() {
                None => {
                    cx.answer_str("Send me a text message.").await?;
                    next(dialogue)
                }
                Some(ans) => dialogue.react(cx, ans).await,
            }
        }
        _ => {
            cx.answer_str("Not allowed").await?;
            next(dialogue)
        }
    }
}

fn user_allowed(user: &User) -> bool {
    let id = user.id.to_string();
    log::info!("id: {}", id);
    let reader = BufReader::new(File::open("allowed_user_ids.txt").expect("file does not exist"));
    reader.lines()
        .any(|line| line.unwrap_or("".parse().unwrap()).eq(id.as_str()))
}
