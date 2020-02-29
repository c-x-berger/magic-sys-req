use std::{boxed::Box, fs};

use irc::client::prelude::*;
#[macro_use]
extern crate lazy_static;

mod bot;
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

fn main() -> irc::error::Result<()> {
    let paths = fs::read_dir("./confs").unwrap();
    let mut reactor = IrcReactor::new()?;
    for path in paths {
        let cfg = Config::load(path.unwrap().path()).unwrap();
        let client = reactor.prepare_client_and_connect(&cfg)?;
        client.identify()?;
        reactor.register_client_with_handler(client, process_message());
    }
    reactor.run()?;
    Ok(())
}

fn process_message() -> impl FnMut(&IrcClient, Message) -> irc::error::Result<()> {
    let mut commands: Vec<Box<dyn BotCommand>> = vec![
        Box::new(bot::Roll {}),
        Box::new(bot::SlashMe {}),
        Box::new(bot::joker::Joker {}),
    ];

    return move |client: &IrcClient, message: Message| {
        if let Command::PRIVMSG(_, msg_txt) = &message.command {
            if let Some(no_prefix) = without_prefix(&msg_txt, &client.current_nickname()) {
                let mut ctx = IrcContext::new(message, client);
                let possible_invokes: Vec<_> = no_prefix.split("and").map(|s| s.trim()).collect();
                // has prefix and is privmsg
                for chance in possible_invokes {
                    for cmd in &mut commands {
                        match cmd.call_if(&chance, &mut ctx) {
                            Ok(_) => continue,
                            Err(_) => eprintln!("error processing message {}", chance),
                        }
                    }
                }
            }
        }
        Ok(())
    };
}
