use irc::client::prelude::{Command, IrcClient, Message};
use regex::Regex;

pub struct IrcContext<'a> {
    message: Message,
    client: &'a IrcClient,
    command: Option<String>,
    noprefix: Option<String>,
}

impl<'a> IrcContext<'a> {
    pub fn new(message: Message, client: &'a IrcClient) -> Self {
        Self {
            message,
            client,
            command: None,
            noprefix: None,
        }
    }

    pub fn get_command(&self) -> &Option<String> {
        &self.command
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
        let cmd = match self.command.as_ref() {
            Some(cmd) => cmd,
            None => return None,
        };
        Some(&self.noprefix.as_ref().unwrap()[cmd.len()..])
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

    callback: Box<dyn FnMut(&IrcContext) + 'a>,
}

impl<'a> BotCommand<'a> {
    pub fn new<CB: FnMut(&IrcContext) + 'a>(aliases: Vec<String>, callback: CB) -> Self {
        Self {
            aliases,
            callback: Box::new(callback),
        }
    }

    pub fn is_call(&self, unprefixed: &str) -> (bool, Option<&str>) {
        for alias in &self.aliases {
            let re = Regex::new(alias).unwrap();
            if re.is_match(unprefixed) {
                return (true, Some(alias));
            }
        }
        (false, None)
    }

    pub fn on_call(&mut self, ctx: &IrcContext) {
        (self.callback)(ctx)
    }

    pub fn call_if(&mut self, unprefixed: &str, ctx: &mut IrcContext) {
        if let (true, Some(cmd)) = self.is_call(unprefixed) {
            ctx.command = Some(cmd.to_string());
            ctx.noprefix = Some(unprefixed.to_string());
            self.on_call(ctx);
        }
    }
}
