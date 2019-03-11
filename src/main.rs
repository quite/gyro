extern crate chrono;
extern crate htmlescape;
extern crate hyper;
extern crate irc;
extern crate regex;
extern crate reqwest;

mod urlinfo;

use chrono::Local;
use irc::client::prelude::*;
use irc::error;
use regex::Regex;

fn has_option(config: &Config, option: &str) -> bool {
    match &config.options {
        Some(options) if options.contains_key(option) => true,
        _ => false,
    }
}

fn get_option<'a>(config: &'a Config, option: &str) -> &'a str {
    config.options.as_ref().unwrap().get(option).unwrap()
}

fn main() {
    let config = Config::load("config.toml").unwrap();

    if !has_option(&config, "proxy") {
        eprintln!("Config is missing required proxy option");
        std::process::exit(1);
    };

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
    if let Command::PRIVMSG(ref _targ, ref msg) = message.command {
        let re = Regex::new(r"(https?://[^-]\S+)").unwrap();
        for cap in re.captures_iter(msg) {
            eprintln!("caught URL: {}", &cap[1]);
            if let Some(target) = message.response_target() {
                client.send_privmsg(
                    target,
                    urlinfo::urlinfo(get_option(client.config(), "proxy"), &cap[1]),
                )?;
            }
        }
    }
    Ok(())
}
