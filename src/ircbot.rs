// ircbot.rs

use anyhow::bail;
use futures::prelude::*;
use irc::client::prelude::*;
use log::*;
use std::{sync::Arc, time::Duration};
use tokio::{sync::RwLock, time::sleep};

use crate::*;

#[derive(Debug)]
pub struct BotConfig {
    pub server: String,
    pub channel: String,
    pub nick: String,
}

impl Default for BotConfig {
    fn default() -> Self {
        BotConfig {
            server: env!("IRC_SERVER").to_string(),
            channel: env!("IRC_CHANNEL").to_string(),
            nick: env!("IRC_NICK").to_string(),
        }
    }
}

impl BotConfig {
    pub fn new() -> anyhow::Result<Self> {
        let config = BotConfig::default();
        debug!("New BotConfig:\n{config:#?}");

        Ok(config)
    }
}

#[derive(Debug)]
pub struct IrcBot {
    irc: Client,
    pub irc_sender: Arc<Sender>,
    pub bot_cfg: Arc<BotConfig>,
    mynick: String,
    msg_nick: String,
    msg_user: String,
    msg_host: String,
    msg_userhost: String,
    halt: bool,
}

impl IrcBot {
    pub async fn new() -> anyhow::Result<Self> {
        let bot_cfg = match BotConfig::new() {
            Ok(b) => b,
            Err(e) => {
                bail!("{e}");
            }
        };

        let irc_cfg = Config {
            server: Some(bot_cfg.server.clone()),
            nickname: Some(bot_cfg.nick.clone()),
            channels: vec![bot_cfg.channel.clone()],
            ..Config::default()
        };

        let irc = match Client::from_config(irc_cfg).await {
            Ok(c) => c,
            Err(e) => {
                bail!("{e}");
            }
        };
        if let Err(e) = irc.identify() {
            bail!("{e}");
        }

        let mynick = irc.current_nickname().to_string();
        let sender = irc.sender();
        Ok(IrcBot {
            irc,
            irc_sender: Arc::new(sender),
            bot_cfg: Arc::new(bot_cfg),
            mynick,
            msg_nick: "NONE".into(),
            msg_user: "NONE".into(),
            msg_host: "NONE".into(),
            msg_userhost: "NONE@NONE".into(),
            halt: false,
        })
    }

    pub fn mynick(&self) -> &str {
        &self.mynick
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        let mut stream = self.irc.stream()?;
        while let Some(message) = stream.next().await.transpose()? {
            trace!("Got msg: {message:?}");
            let mynick = self.mynick().to_string();

            let msg_nick;
            let msg_user;
            let msg_host;

            if let Some(Prefix::Nickname(nick, user, host)) = message.prefix {
                (msg_nick, msg_user, msg_host) = (nick, user, host);
            } else {
                (msg_nick, msg_user, msg_host) = ("NONE".into(), "NONE".into(), "NONE".into());
            }
            self.msg_nick = msg_nick.clone();
            self.msg_user = msg_user.clone();
            self.msg_host = msg_host.clone();
            let userhost = format!("{msg_user}@{msg_host}");
            self.msg_userhost = userhost.clone();

            match message.command {
                Command::Response(resp, v) => {
                    debug!("Got response type {resp:?} contents: {v:?}");
                }

                Command::PRIVMSG(channel, msg) => {
                    // debug!("PRIVMSG <{channel}> {msg}");

                    if channel == mynick && msg_nick == env!("IRC_OWNER_NICK") {
                        // we received a private message from the owner
                        if msg.starts_with("say ") {
                            self.irc
                                .send_privmsg(&self.bot_cfg.channel, &msg[4..msg.len()])
                                .ok();
                        } else if msg == "kill" {
                            self.halt = true;
                            bail!("Received kill command, exiting .");
                        }
                    }
                }

                Command::NICK(newnick) => {
                    debug!(
                        "NICK: {msg_nick} USER: {msg_user} HOST: {msg_host} NEW NICK: {newnick}"
                    );
                    if msg_nick == *mynick {
                        info!("My NEW nick: {newnick}");
                        self.mynick = newnick;
                    }
                }

                cmd => {
                    debug!("Unhandled command (ignored): {cmd:?}")
                }
            }
        }

        Ok(())
    }
}

pub async fn run_bot(state: Arc<RwLock<MyState>>) -> anyhow::Result<()> {
    let mut first_time = true;
    loop {
        if first_time {
            first_time = false;
        } else {
            error!("Sleeping 10s...");
            sleep(Duration::from_secs(10)).await;
            error!("Retrying start");
        }

        let mut ircbot = IrcBot::new().await?;
        {
            let mut st = state.write().await;
            st.snd = Some(ircbot.irc_sender.clone());
            st.bcfg = Some(ircbot.bot_cfg.clone());
        }
        if let Err(e) = ircbot.run().await {
            error!("{e}");
        }
        {
            let mut st = state.write().await;
            st.snd = None;
            st.bcfg = None;
            if ircbot.halt {
                drop(ircbot);
                error!("Kill switch activated, halt now.");
                panic!("halt");
            } else {
                drop(ircbot);
            }
        }
    }
}

// EOF
