use std::{
    fs::File,
    io,
    io::{BufRead, BufReader},
};

use rand::seq::SliceRandom;
use regex::Regex;

use crate::command::{BotCommand, IrcContext};

fn read_jokes(file: &str) -> io::Result<Vec<String>> {
    let f = File::open(file)?;
    let reader = BufReader::new(f);
    Ok(reader.lines().map(|r| r.unwrap()).collect())
}

lazy_static! {
    static ref JOKES: Vec<String> = read_jokes("jokes.txt").expect("could not read jokes");
}

pub struct Joker {}

impl BotCommand for Joker {
    fn is_call<'a>(&self, invoke: &'a str) -> (bool, Option<&'a str>) {
        lazy_static! {
            static ref RE: Regex =
                Regex::new("tell(?: (me|us))? a joke").expect("could not compile regex");
        }

        if let Some(m) = RE.find(invoke) {
            return (true, Some(m.as_str()));
        }
        (false, None)
    }

    fn on_call(&mut self, ctx: &IrcContext) -> irc::error::Result<()> {
        let mut rng = rand::thread_rng();

        ctx.send(match JOKES.choose(&mut rng) {
            Some(j) => j,
            None => "i forgot all my jokes",
        })?;

        Ok(())
    }
}
