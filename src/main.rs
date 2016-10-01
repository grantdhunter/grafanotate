#![feature(custom_derive, plugin)]
#![plugin(serde_macros)]

extern crate slack;
extern crate chrono;
extern crate hyper;
extern crate serde_json;
extern crate toml;
extern crate rustc_serialize;

use chrono::{UTC, TimeZone, Local};
use std::io::Read;
use std::fs::File;
use hyper::Client;
use std::process;

struct EventHandler {
    config: Config,
}


impl EventHandler {
    fn new(config: Config) -> EventHandler {
        EventHandler { config: config }
    }
}

impl slack::EventHandler for EventHandler {
    fn on_event(&mut self,
                client: &mut slack::RtmClient,
                event: Result<&slack::Event, slack::Error>,
                _: &str) {
        let event = event.unwrap();

        let message = match *event {
            slack::Event::Message(ref m) => m,
            _ => return,
        };

        let txt;
        let chan;

        match *message {
            slack::Message::Standard {
                ts: _,
                user: _,
                is_starred: _,
                pinned_to: _,
                reactions: _,
                edited: _,
                attachments: _,
                ref text,
                ref channel
            } => {
                txt = text.clone().unwrap();
                chan = channel.clone().unwrap();
            }
            _ => return,
        };

        let bot_tag = format!("<@{}>", client.get_id().unwrap());
        let cmd = parse_command(&txt, &bot_tag);


        let reply = match cmd {
            Command::Annotate(annotate) => {
                save(&self.config, &annotate);
                "Done! Annotation added."
            }
            Command::Help => {
                "Type your annotation in \"title. tag 1,tag 2, tag 3. time.\" or \n  \"title. tag \
                 1,tag 2, tag 3.\""
            }
            Command::None => "Sorry, I don't know what you want",
        };

        let _ = client.send_message(&chan, &reply);
    }

    fn on_ping(&mut self, _: &mut slack::RtmClient) {}

    fn on_close(&mut self, _: &mut slack::RtmClient) {}

    fn on_connect(&mut self, _: &mut slack::RtmClient) {}
}


#[derive(Debug)]
enum Command {
    Help,
    Annotate(Annotate),
    None,
}


#[derive(Debug, Serialize, Deserialize)]
struct Annotate {
    what: String,
    tags: Vec<String>,
    when: i64,
}


impl Annotate {
    fn new(what: &String, tags: &String, when: &Option<&String>) -> Annotate {
        Annotate {
            what: what.clone().trim().to_string(),
            tags: tags.split(",").map(|s| s.trim().to_string()).collect::<Vec<String>>(),
            when: when.and_then(|s| {
                let date = Local.datetime_from_str(s, "%F %R");

                match date {
                    Ok(date) => Some(date.with_timezone(&UTC)),
                    Err(_) => Some(UTC::now()),
                }
            })
                .or_else(|| Some(UTC::now()))
                .and_then(|d| Some(d.timestamp() * 1000))
                .unwrap(),
        }
    }
}


fn parse_command(message: &String, bot_tag: &String) -> Command {
    let tokens: Vec<String> = message.split(".").map(|s| s.trim().to_string()).collect();

    let cmd_token = match tokens.get(0) {
        Some(s) => s.trim_left_matches(bot_tag).to_string(),
        None => return Command::None,
    };

    match cmd_token.as_ref() {
        "help" => Command::Help,
        _ => {
            if tokens.len() >= 2 {
                return Command::Annotate(Annotate::new(&cmd_token,
                                                       &tokens.get(1).unwrap(),
                                                       &tokens.get(2)));
            } else {
                return Command::None;
            }
        }
    }
}

fn save(config: &Config, annotate: &Annotate) {
    let client = Client::new();

    let body = serde_json::to_string(annotate).unwrap();

    let resp = client.post(config.url.as_str())
        .body(body.as_str())
        .send();

    match resp {
        Ok(_) => {},
        Err(err) => println!("{:?}", err),
    }
}


#[derive(RustcDecodable, Clone)]
struct Config {
    slack_key: String,
    url: String,
}


fn main() {
    let mut config = String::new();
    let _ = File::open("config.toml")
        .and_then(|mut f| f.read_to_string(&mut config))
        .map_err(|e| {
            println!("{}", e);
            process::exit(1);
        });

    let mut parser = toml::Parser::new(&config);

    let parsed = parser.parse().unwrap();
    let config_parsed = parsed.get("config").unwrap();

    let config = toml::decode::<Config>(config_parsed.clone()).unwrap();

    let mut event_handler = EventHandler::new(config.clone());

    let mut cli = slack::RtmClient::new(&config.slack_key.clone());

    let result = cli.login_and_run(&mut event_handler);

    match result {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
