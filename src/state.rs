// state.rs

use irc::client::prelude::*;
use std::sync::Arc;

use crate::*;

#[derive(Debug, Default)]
pub struct MyState {
    pub cnt: u64,
    pub snd: Option<Arc<Sender>>,
    pub bcfg: Option<Arc<BotConfig>>,
    pub halt: bool,
}
impl MyState {
    pub fn new() -> Self {
        MyState::default()
    }
}

// EOF
