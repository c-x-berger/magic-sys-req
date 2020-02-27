use std::{
    fs,
    fs::File,
    io::{prelude::*, BufReader},
};

use irc::client::prelude::*;
use rand::seq::SliceRandom;

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

fn read_jokes() -> std::io::Result<Vec<String>> {
    let file = File::open("jokes.txt")?;
    let reader = BufReader::new(file);
    Ok(reader.lines().map(|l| l.expect("what the?")).collect())
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
    let jokes = read_jokes().expect("could not read jokes");

    let mut commands = Vec::new();

    let echo_cmd = BotCommand::new(
        vec!["echo ".to_string(), "repeat ".to_string()],
        move |ctx| {
            ctx.get_client().send_privmsg(
                ctx.get_message().response_target().unwrap(),
                ctx.command_params_str().unwrap(),
            )
        },
    );
    commands.push(echo_cmd);

    let joker = BotCommand::new(vec!["tell(?: (me|us))? a joke".to_string()], move |ctx| {
        let mut rng = rand::thread_rng();
        ctx.get_client().send_privmsg(
            ctx.get_message().response_target().unwrap(),
            &jokes.choose(&mut rng).unwrap(),
        )
    });
    commands.push(joker);

    let dice = BotCommand::new(vec!["[Rr]oll ".to_string()], bot::roll_ndn);
    commands.push(dice);
    let slash_me = BotCommand::new(vec![r"/me".to_string()], bot::do_action);
    commands.push(slash_me);

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
