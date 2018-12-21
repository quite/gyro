extern crate chrono;
extern crate irc;
extern crate regex;
extern crate reqwest;
extern crate select;

use chrono::Local;
use irc::client::prelude::*;
use irc::error;
use regex::Regex;

mod urlinfo;

fn main() {
    let config = Config::load("config.toml").unwrap();
    let mut reactor = IrcReactor::new().unwrap();

    loop {
        let result = reactor
            .prepare_client_and_connect(&config)
            .and_then(|client| client.identify().and(Ok(client)))
            .and_then(|client| {
                reactor.register_client_with_handler(client, process_msg);
                Ok(())
            })
            .and_then(|()| reactor.run());
        match result {
            Ok(_) => break,
            Err(e) => {
                let now = Local::now();
                eprintln!("{} {}", now.format("%FT%T"), e);
            }
        }
    }
}

fn process_msg(client: &IrcClient, message: Message) -> error::Result<()> {
    print!("{}", message);
    match message.command {
        Command::PRIVMSG(ref _targ, ref msg) => {
            let re = Regex::new(r"(https?://\S+)").unwrap();
            for cap in re.captures_iter(msg) {
                println!("caught URL: {}", &cap[1]);
                if let Some(t) = message.response_target() {
                    let info = match urlinfo::urlinfo(&cap[1]) {
                        Ok(txt) => format!("`{}`", txt),
                        Err(txt) => txt.to_string(),
                    };
                    client.send_privmsg(t, info)?;
                }
            }
        }
        _ => (),
    }
    Ok(())
}
