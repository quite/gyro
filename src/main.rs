extern crate irc;

use irc::client::prelude::*;
use irc::error;

fn main() {
    let config = Config::load("config.toml").unwrap();
    let mut reactor = IrcReactor::new().unwrap();
    let client = reactor.prepare_client_and_connect(&config).unwrap();
    client.identify().unwrap();
    reactor.register_client_with_handler(client, process_msg);
    reactor.run().unwrap();
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
