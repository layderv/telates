use teloxide_macros::Transition;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use chrono::{DateTime, Utc};
use teloxide::types::User;
use std::fmt::{Debug, Formatter};
use std::fmt;

#[derive(Debug, Deserialize, From, Serialize, Transition)]
pub enum Dialogue {
    Start(StartState),
    Run(RunState),
}

impl Default for Dialogue {
    fn default() -> Self {
        Self::Start(StartState)
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StartState;

type UserId = i32;

#[derive(Deserialize, Serialize)]
pub struct RunState {
    pub owner: UserId,
    pub chat_id: i64,
    pub subscriptions: Vec<String>,
    pub refresh_rate: Duration,
    pub saved: Vec<u64>,
    pub last_msg_id: Option<u64>,
    pub last_refresh: Option<DateTime<Utc>>,
}

impl RunState {
    pub fn new(owner: User, chat_id: i64) -> RunState {
        RunState {
            owner: owner.id,
            chat_id,
            subscriptions: vec![],
            refresh_rate: Duration::from_secs(15 * 60),
            saved: vec![],
            last_msg_id: None,
            last_refresh: None,
        }
    }
}

impl Debug for RunState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("RunState")
            .field("owner", &self.owner)
            .field("chat_id", &self.chat_id)
            .field("subscriptions", &self.subscriptions)
            .field("refresh_rate", &self.refresh_rate)
            .field("saved", &self.saved)
            .field("last_msg_id", &self.last_msg_id)
            .field("last_refresh", &self.last_refresh)
            .finish()
    }
}

