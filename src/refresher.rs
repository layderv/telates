use teloxide::Bot;
use redis::{Connection, Commands};
use chrono::{DateTime, Utc};
use crate::states::{RunState, Dialogue};
use std::time::Duration;
use std::thread;
use teloxide::dispatching::dialogue::RedisStorageError;
use rss::Channel;
use std::error::Error;
use teloxide::prelude::Request;
use crate::client::Client;

pub struct Refresher {
    bot: Bot,

    conn: Connection,
    last_run: DateTime<Utc>,
}

impl Refresher {
    pub fn new(bot: Bot) -> Refresher {
        let client = redis::Client::open("redis://127.0.0.1/").expect("is redis running?");
        let conn = client.get_connection().expect("cannot connect to redis");
        
        Refresher {
            bot,
            conn,
            last_run: Utc::now(),
        }
    }

    #[tokio::main]
    pub async fn run(&mut self) -> ! {
        log::info!("refresher running");
        loop {
            let keys: Vec<Vec<u8>> = self.conn.scan::<Vec<u8>>()
                .expect("cannot retrieve keys from redis")
                .collect();
            for key in keys {
                let s: String = std::str::from_utf8(&*key)
                    .map(|s| String::from(s))
                    .unwrap_or(String::from(format!("{:?}", key)));
                match self.get_state(key.clone()) {
                    Ok(state) => {
                        match self.refresh(&state).await {
                            Ok(_) => (),
                            Err(e) => log::error!("error refreshing: {:?}. {}", state, e)
                        }
                        log::info!("refreshed for: {}", s)
                    },
                    _ => log::error!("refresh error for: {}", s),
                }
            }

            self.last_run = Utc::now();
            thread::sleep(Duration::from_secs(15 * 60)); // 15 min
        }
    }

    fn get_state(&mut self, key: Vec<u8>) -> Result<RunState, RedisStorageError<i64>> {
        let deserialized: Option<Dialogue> = self
            .conn
            .get::<Vec<u8>, Option<Vec<u8>>>(key).unwrap_or(None)
            .map(|d| bincode::deserialize(&d).map_err(RedisStorageError::SerdeError))
            .transpose()
            .unwrap_or(None);
        log::debug!("deserialized: {:X?}", deserialized);
        if let Some(d) = deserialized {
            if let Dialogue::Run(state) = d {
                return Ok(state)
            }
        }
        Err(RedisStorageError::SerdeError(0))
    }

    async fn refresh(&mut self, state: &RunState) -> Result<(), Box<dyn Error>> {
        let chat_id = state.chat_id;
        for url in &state.subscriptions {
            match self.get_channel(&url) {
                Ok(channel) => {
                    log::debug!("ok for {:?}", channel.link);
                    let b = self.bot.clone();
                    let last_run = self.last_run;
                    tokio::task::spawn(async move {
                        match Refresher::send_messages(b, channel, last_run, chat_id).await {
                            Ok(_) => (),
                            Err(e) => log::error!("error refreshing for chat_id: {}. {}", chat_id, e),
                        }
                    }).await?;
                }
                Err(_) => {
                    log::error!("error refreshing {}", url);
                }
            }
        }
        Ok(())
    }

    async fn send_messages(bot: Bot, channel: Channel, last_run: DateTime<Utc>, chat_id: i64) -> Result<(), Box<dyn Error>> {
        for item in channel.items {
            if let Some(date) = item.pub_date {
                if let Ok(pub_date) = DateTime::parse_from_rfc2822(date.as_str()) {
                    let pub_date_utc: DateTime<Utc> = pub_date.with_timezone(&Utc);
                    if pub_date_utc > last_run {
                        let link = item.link.unwrap_or(String::from("<no link>"));
                        let descr: String = item.description
                            .map(|s| {
                                s.chars().take(30).collect::<String>() +
                                    if s.len() > 30 { "..." } else { "" }
                            })
                            .unwrap_or(String::from("<no description>"));
                        let msg = format!("{}: {} - {}",
                            item.title.unwrap_or(String::from("<no title>")),
                            descr,
                            link,

                        );
                        log::debug!("sending <{}> to <{}>", link, chat_id);
                        bot.send_message(chat_id, msg).send().await?;
                    }
                }
            }
        }
        Ok(())
    }

    fn get_channel(&mut self, url: &String) -> Result<Channel, Box<dyn Error>> {
        let content = Client::new(std::time::Duration::from_secs(5))
            .client()
            .get(url)
            .send()?
            .bytes()?;
        Ok(Channel::read_from(&content[..])?)
    }
}
