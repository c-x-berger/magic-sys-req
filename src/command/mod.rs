use std::io;

use irc::{
    client::prelude::*,
    error::{IrcError, Result},
};

pub struct IrcContext<'a, 'b> {
    message: Message,
    client: &'a IrcClient,
    alias_used: Option<&'b str>,
    invokation: Option<&'b str>,
}

impl<'a> IrcContext<'a, '_> {
    pub fn new(message: Message, client: &'a IrcClient) -> Self {
        Self {
            message,
            client,
            alias_used: None,
            invokation: None,
        }
    }

    pub fn get_alias_used(&self) -> Option<&str> {
        self.alias_used
    }

    pub fn get_message(&self) -> &Message {
        &self.message
    }

    pub fn message_content(&self) -> Option<&str> {
        if let Command::PRIVMSG(_, message) = &self.message.command {
            return Some(message);
        }
        None
    }

    pub fn command_params_str(&self) -> Option<&str> {
        let alias = match self.get_alias_used() {
            Some(cmd) => cmd,
            None => return None,
        };
        let ret = &self.invokation.as_ref().unwrap()[alias.len()..];
        match ret.len() {
            0 => None,
            _ => Some(ret),
        }
    }

    pub fn command_params(&self) -> Option<Vec<&str>> {
        match self.command_params_str() {
            Some(params) => Some(params.split_ascii_whitespace().collect()),
            None => None,
        }
    }

    pub fn get_client(&self) -> &IrcClient {
        self.client
    }

    pub fn send(&self, msg: &str) -> Result<()> {
        let resp_target = match self.get_message().response_target() {
            Some(target) => target,
            None => {
                return Err(IrcError::Io(io::Error::new(
                    io::ErrorKind::NotFound,
                    "no real response target",
                )))
            }
        };
        self.get_client().send_privmsg(resp_target, msg)?;
        Ok(())
    }
}

pub trait BotCommand {
    // type Call<'a> = (bool, Option<&'a str>);
    // for some reason static methods make trait objects impossible
    fn is_call<'a>(&self, invokation: &'a str) -> (bool, Option<&'a str>);
    fn on_call(&mut self, ctx: &IrcContext) -> Result<()>;

    fn call_if<'i>(&mut self, invokation: &'i str, ctx: &mut IrcContext<'_, 'i>) -> Result<()> {
        if let (true, Some(alias)) = self.is_call(invokation) {
            ctx.invokation = Some(invokation);
            ctx.alias_used = Some(alias);
            self.on_call(ctx)?;
        }
        Ok(())
    }
}
