use irc::{client::prelude::*, error::Result};
use rand::distributions::{Distribution, Uniform};
use regex::Regex;

use crate::command::{BotCommand, IrcContext};

pub mod joker;

fn bad_parse_int(val: &str) -> u32 {
    match val.parse() {
        Ok(int) => int,
        Err(_) => 1,
    }
}

pub struct Roll {}

impl BotCommand for Roll {
    fn is_call<'a>(&self, invoke: &'a str) -> (bool, Option<&'a str>) {
        lazy_static! {
            static ref RE: Regex = Regex::new("[Rr]oll").expect("regex bad");
        }
        if let Some(m) = RE.find(invoke) {
            return (true, Some(m.as_str()));
        }
        (false, None)
    }

    fn on_call(&mut self, ctx: &IrcContext) -> Result<()> {
        // parse dice
        let dice_string = match ctx.command_params() {
            Some(params) => params[0],
            None => "1d6",
        };
        let tokens: Vec<_> = dice_string.split("d").collect();
        let ndn = (
            bad_parse_int(tokens.get(0).unwrap_or(&"1")),
            bad_parse_int(tokens.get(1).unwrap_or(&"6")),
        );
        // roll
        let mut rng = rand::thread_rng();
        let die = Uniform::from(1..ndn.1 + 1);
        let mut rolls = 0;
        let mut rolled: Vec<u32> = Vec::new();
        while rolls < ndn.0 {
            rolls += 1;
            rolled.push(die.sample(&mut rng));
        }
        let mut send = format!("Rolling {}d{}. ", ndn.0, ndn.1);
        send.push_str(&format!(
            "You rolled: {} = {}",
            rolled
                .iter()
                .map(|i| i.to_string())
                .collect::<Vec<_>>()
                .join(" + "),
            rolled.iter().sum::<u32>()
        ));
        ctx.send(&send)?;
        Ok(())
    }
}

pub struct SlashMe {}

impl BotCommand for SlashMe {
    fn is_call<'a>(&self, invoke: &'a str) -> (bool, Option<&'a str>) {
        lazy_static! {
            static ref RE: Regex = Regex::new("/me").expect("could not compile regex");
        }
        if let Some(m) = RE.find(invoke) {
            return (true, Some(m.as_str()));
        }
        (false, None)
    }

    fn on_call(&mut self, ctx: &IrcContext) -> Result<()> {
        let act = match ctx.command_params_str() {
            Some(act) => act.trim().to_string(),
            None => {
                let nick = ctx.get_message().source_nickname().unwrap_or("c-x-berger");
                format!("slaps {} around a bit with a large trout", nick)
            }
        };
        let dest = ctx.get_message().response_target().unwrap();
        ctx.get_client().send_action(dest, act)?;
        Ok(())
    }
}

pub fn get_fcc_info(ctx: &IrcContext) -> Result<()> {
    let search_term = match ctx.command_params() {
        Some(params) => params[0],
        None => {
            ctx.send("no search term given")?;
            return Ok(());
        }
    };

    Ok(())
}
