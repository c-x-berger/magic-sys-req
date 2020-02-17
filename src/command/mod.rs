use irc::client::prelude::{Command, IrcClient, Message};
use irc::error::Result;
use regex::Regex;

pub struct IrcContext<'a> {
    message: Message,
    client: &'a IrcClient,
    alias_used: Option<String>,
    invokation: Option<String>,
}

impl<'a> IrcContext<'a> {
    pub fn new(message: Message, client: &'a IrcClient) -> Self {
        Self {
            message,
            client,
            alias_used: None,
            invokation: None,
        }
    }

    pub fn get_alias_used(&self) -> Option<&String> {
        self.alias_used.as_ref()
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
}

pub struct BotCommand<'a> {
    aliases: Vec<String>,
    callback: Box<dyn FnMut(&IrcContext) -> Result<()> + 'a>,
}

impl<'a> BotCommand<'a> {
    pub fn new<CB: FnMut(&IrcContext) -> Result<()> + 'a>(
        aliases: Vec<String>,
        callback: CB,
    ) -> Self {
        Self {
            aliases,
            callback: Box::new(callback),
        }
    }

    pub fn is_call<'b>(&self, invokation: &'b str) -> (bool, Option<&'b str>) {
        for alias in &self.aliases {
            let re = Regex::new(alias).unwrap();
            match re.find(invokation) {
                Some(match_) => return (true, Some(match_.as_str())),
                None => continue,
            }
        }
        (false, None)
    }

    pub fn on_call(&mut self, ctx: &IrcContext) -> Result<()> {
        (self.callback)(ctx)
    }

    pub fn call_if(&mut self, invokation: &str, ctx: &mut IrcContext) -> Result<()> {
        if let (true, Some(alias)) = self.is_call(invokation) {
            ctx.alias_used = Some(alias.to_string());
            ctx.invokation = Some(invokation.to_string());
            return self.on_call(ctx);
        }
        Ok(())
    }
}
