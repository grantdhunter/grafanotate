extern crate slack;
extern crate chrono;

use chrono::{DateTime, UTC, TimeZone, Local};

struct EventHandler;


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
            slack::Message::Standard(stMsg) => {
                txt = stMsg.text.clone().unwrap();
                chan = stMsg.channel.clone().unwrap();
            }
            _ => return,
        };

        let cmd = parse_command(txt);


        let reply = match cmd {
            Command::Annotate(Annotate) => "Done! Annotation added.",
            Command::Help => {
                "Type your annotation in \"title. tag 1,tag 2, tag 3. time.\" or \n  \"title. tag \
                 1,tag 2, tag 3.\""
            }
            Command::None => "Sorry, I don't know what you want",
        };

        let _ = client.send_message(&chan, &reply);

        println!("{:?}", cmd);

    }

    fn on_ping(&mut self, _: &mut slack::RtmClient) {
        println!("on_ping");
    }

    fn on_close(&mut self, _: &mut slack::RtmClient) {
        println!("on_close");
    }

    fn on_connect(&mut self, _: &mut slack::RtmClient) {
        println!("Connected");
    }
}



#[derive(Debug)]
enum Command {
    Help,
    Annotate(Annotate),
    None,
}


#[derive(Debug)]
struct Annotate {
    title: String,
    tags: Vec<String>,
    time: DateTime<UTC>,
}


impl Annotate {
    fn new(&title: &String, &tags: &String, &time: &Option<&String>) -> Annotate {
        Annotate {
            title: title,
            tags: tags.split(", ").map(|s| s.to_string()).collect::<Vec<String>>(),
            time: time.and_then(|s| {

                    let date = Local.datetime_from_str(s, "%F %R");

                    match date {
                        Ok(date) => Some(date.with_timezone(&UTC)),
                        Err(err) => {
                            println!("time error: {:?} using current time.", err);
                            Some(UTC::now())
                        }
                    }

                })
                .or_else(|| Some(UTC::now()))
                .unwrap(),
        }
    }
}



fn parse_command(message: String) -> Command {
    let tokens: Vec<String> = message.split(". ").map(|s| s.to_string()).collect();

    let cmd_token = match tokens.get(0) {
        Some(s) => s,
        None => return Command::None,

    };

    match cmd_token.as_ref() {
        "help" => Command::Help,
        _ => {
            if tokens.len() >= 2 {
                return Command::Annotate(Annotate::new(&tokens.get(0).unwrap(),
                                                       &tokens.get(1).unwrap(),
                                                       &tokens.get(2)));
            } else {
                return Command::None;
            }
        }
    }
}







fn main() {
    let api_key = "xoxb-60267315107-BM9hS0cOYPThDVLdHg8OPn4u".to_string();

    let mut event_handler = EventHandler;

    let mut cli = slack::RtmClient::new(&api_key);

    let result = cli.login_and_run(&mut event_handler);

    match result {
        Ok(_) => {}
        Err(err) => panic!("Error: {}", err),
    }
}
