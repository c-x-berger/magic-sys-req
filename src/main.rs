use std::{
    fs::File,
    io::{prelude::*, BufReader},
};

use irc::client::prelude::*;
use rand::seq::SliceRandom;

mod bot;
mod command;

use bot::roll_ndn;
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

fn read_jokes() -> std::io::Result<Vec<String>> {
    let file = File::open("jokes.txt")?;
    let reader = BufReader::new(file);
    Ok(reader.lines().map(|l| l.expect("what the?")).collect())
}

fn main() {
    let mut rng = rand::thread_rng();
    let jokes = read_jokes().expect("could not read jokes");

    let mut commands: Vec<BotCommand> = Vec::new();

    let echo_cmd = BotCommand::new(vec!["echo".to_string(), "repeat".to_string()], |ctx| {
        ctx.get_client().send_privmsg(
            ctx.get_message().response_target().unwrap(),
            ctx.command_params_str().unwrap(),
        )
    });
    commands.push(echo_cmd);

    let joker = BotCommand::new(vec!["tell(?: (me|us))? a joke".to_string()], |ctx| {
        ctx.get_client().send_privmsg(
            ctx.get_message().response_target().unwrap(),
            &jokes.choose(&mut rng).unwrap(),
        )
    });
    commands.push(joker);

    let dice = BotCommand::new(vec!["[Rr]oll".to_string()], roll_ndn);
    commands.push(dice);

    let client = IrcClient::new("config.toml").unwrap();
    client.identify().unwrap();

    client
        .for_each_incoming(|irc_msg| {
            if let Command::PRIVMSG(_, message) = &irc_msg.command {
                if let Some(unpre) = without_prefix(&message, &client.current_nickname()) {
                    let mut ctx = IrcContext::new(irc_msg, &client);
                    for cmd in &mut commands {
                        match cmd.call_if(&unpre, &mut ctx) {
                            Ok(_) => continue,
                            Err(_) => println!("encountered error processing {}", unpre),
                        };
                    }
                }
            }
        })
        .unwrap();
}
