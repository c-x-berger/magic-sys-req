use irc::client::prelude::*;
use irc::error::Result;
use rand::distributions::{Distribution, Uniform};

use crate::command::IrcContext;

fn bad_parse_int(val: &str) -> u32 {
    match val.parse() {
        Ok(int) => int,
        Err(_) => 1,
    }
}

pub fn roll_ndn(ctx: &IrcContext) -> Result<()> {
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
    ctx.get_client().send_privmsg(
        ctx.get_message().response_target().unwrap(),
        format!("Rolling {}d{}", ndn.0, ndn.1),
    )?;
    // roll
    let mut rng = rand::thread_rng();
    let die = Uniform::from(1..ndn.1 + 1);
    let mut rolls = 0;
    let mut rolled: Vec<u32> = Vec::new();
    while rolls < ndn.0 {
        rolls += 1;
        rolled.push(die.sample(&mut rng));
    }
    let send = format!(
        "You rolled: {} = {}",
        rolled
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join(" + "),
        rolled.iter().sum::<u32>()
    );
    ctx.get_client()
        .send_privmsg(ctx.get_message().response_target().unwrap(), send)?;
    Ok(())
}
