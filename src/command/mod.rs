use irc::client::prelude::{Command, IrcClient, Message};

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

pub struct BotCommand {
    aliases: Vec<String>,

    callback: fn(&IrcContext),
}

impl BotCommand {
    pub fn new(aliases: Vec<String>, callback: fn(&IrcContext)) -> Self {
        Self { aliases, callback }
    }

    pub fn is_call(&self, unprefixed: &str) -> (bool, Option<&str>) {
        for prefix in &self.aliases {
            if unprefixed.starts_with(prefix) {
                return (true, Some(prefix));
            }
        }
        (false, None)
    }

    pub fn on_call(&self, ctx: &IrcContext) {
        (self.callback)(ctx)
    }

    pub fn call_if(&self, unprefixed: &str, ctx: &mut IrcContext) {
        if let (true, Some(cmd)) = self.is_call(unprefixed) {
            ctx.command = Some(cmd.to_string());
            ctx.noprefix = Some(unprefixed.to_string());
            self.on_call(ctx);
        }
    }
}
