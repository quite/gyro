extern crate chrono;
extern crate irc;

use chrono::Local;
use irc::client::prelude::*;
use irc::error;

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
            }).and_then(|()| reactor.run());
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
            if msg.contains("gyro") {
                match message.response_target() {
                    None => (),
                    Some(ref t) => client.send_privmsg(t, "Hej!")?,
                }
            }
        }
        _ => (),
    }
    Ok(())
}
