use teloxide::prelude::*;
use teloxide_macros::teloxide;
use crate::states::*;
use url::Url;
use rss::Channel;
use regex::Regex;
use std::lazy::SyncLazy;
use bytes::Bytes;
use crate::client::Client;
use rss::validation::Validate;

static INTERNAL_IP_SPACE_RE: SyncLazy<Regex> = SyncLazy::new(||
    Regex::new(r"/(^127\.)|(^192\.168\.)|(^10\.)|(^172\.1[6-9]\.)|(^172\.2[0-9]\.)|(^172\.3[0-1]\.)|(^::1$)|(^[fF][cCdD])/").unwrap()
);

#[teloxide(subtransition)]
async fn start(_state: StartState, cx: TransitionIn, _ans: String) -> TransitionOut<Dialogue> {
    cx.answer_str("Welcome").await?;
    log::info!("new chat: {}", cx.update.chat.id);
    next(
        RunState::new(
            cx.update.from().expect("user not found").clone(),
            cx.chat_id()
        )
    )
}

#[teloxide(subtransition)]
async fn run(mut state: RunState, cx: TransitionIn, ans: String) -> TransitionOut<Dialogue> {
    if ans.is_empty() {
        return next(state)
    }

    let cmd: Vec<&str> = ans.split(" ").collect();
    match cmd[0] {
        "/subscribe" => {
            let url = cmd.get(1);
            match url {
                Some(url) if validate_url(*url).await && state.subscriptions.len() < 10 => {
                    state.subscriptions.push(String::from(*url));
                    log::debug!("subscribed to: {}", *url);
                    cx.answer_str(format!("subscribed to: {}", url)).await?;
                }
                _ => {
                    log::debug!("{}", format!("invalid url: {}", cmd.get(1).unwrap_or(&"")));
                    cx.answer_str("invalid url").await?;
                }
            }
        }
        "/save" => {
            match cmd.get(1).map(|n| n.parse::<usize>()) {
                Some(Ok(id)) if id < state.last_msg_id.unwrap_or(0) => {
                    state.saved.push(id);
                    cx.answer_str("saved!").await?;
                }
                _ => {
                    cx.answer_str("invalid id").await?;
                },
            }
        }
        "/subscriptions" => {
            let subs = state.subscriptions.iter()
                .enumerate()
                .map(|(idx, item)| format!("{}: {}", idx, item))
                .collect::<Vec<String>>()
                .join("\n");
            cx.answer_str(format!("your subscriptions: \n{}", subs)).await?;
        }
        "/unsubscribe" => {
            match cmd.get(1).map(|n| n.parse::<usize>()) {
                Some(Ok(id)) => {
                    state.subscriptions.remove(id);
                    cx.answer_str("removed").await?;
                }
                _ => {
                    cx.answer_str("Usage: /unsubscribe <id>").await?;
                }
            }
        }
        _ => ()
    }
    next(state)
}

async fn validate_url(input_url: &str) -> bool {
    match Url::parse(input_url) {
        Ok(url) => {
            let host = url.host_str().unwrap_or("");
            let valid = (url.scheme().eq("http") || url.scheme().eq("https")) &&
                url.has_host() &&
                !INTERNAL_IP_SPACE_RE.is_match(host) &&
                !host.eq("localhost");
            if valid {
                let valid_rss = |content: bytes::Bytes|
                    Channel::read_from(&content[..])
                        .map(|ch| ch.validate());
                if let Ok(content) = can_fetch_url(input_url) {
                    return valid_rss(content).is_ok()
                }
            }
            false
        }
        _ => false,
    }
}

fn can_fetch_url(url: &str) -> reqwest::Result<Bytes> {
    let mut client = Client::new(std::time::Duration::from_secs(10));
    client.client()
        .get(url)
        .send()?
        .bytes()
}
