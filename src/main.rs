use irc::client::prelude::*;

mod command;

use command::{BotCommand, IrcContext};

const PREFIXES: &[&str] = &[":", ",", " "];

fn without_prefix(message: &str, nick: &str) -> Option<String> {
    for fix in PREFIXES {
        let test = format!("{}{}", nick, fix);
        if message.starts_with(&test) {
            let ret = &message[test.len()..];
            return Some(ret.trim().to_string());
        }
    }
    None
}

fn main() {
    let echo_cmd = BotCommand::new(vec!["echo".to_string(), "repeat".to_string()], |ctx| {
        ctx.get_client()
            .send_privmsg(
                ctx.get_message().response_target().unwrap(),
                ctx.command_params_str().unwrap(),
            )
            .unwrap();
    });

    let client = IrcClient::new("config.toml").unwrap();
    client.identify().unwrap();

    client
        .for_each_incoming(|irc_msg| {
            if let Command::PRIVMSG(_channel, message) = &irc_msg.command {
                if let Some(unpre) = without_prefix(&message, &client.current_nickname()) {
                    let mut ctx = IrcContext::new(irc_msg, &client);
                    echo_cmd.call_if(&unpre, &mut ctx);
                }
            }
        })
        .unwrap();
}
