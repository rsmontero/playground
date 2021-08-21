mod listener;
mod messages;

use crate::messages::{APIMessage, APIMessageTypes, Message};

use crate::listener::Streamer;

use std::{
    collections::HashMap,
    thread,
    time
};

fn main() {
    let msg1 = String::from("Red 124 SUCCESS aGVsbG8gd29ybGQ=");

    let msg1 = APIMessage::parse(&msg1).unwrap();

    println!("{:?}", msg1);

    let mut api_str: Streamer<APIMessage> = Streamer {
        actions: HashMap::new(),
    };

    api_str.register(APIMessageTypes::Red, |m: APIMessage| {
        let s: String = m.to_string();
        println!("Callback MSG: {}", s);
    });

    api_str.do_action(msg1, true);

    let msg2 = String::from("Yellow 124 SUCCESS aGVsbG8gd29ybGQ=");

    api_str.do_action(APIMessage::parse(&msg2).unwrap(), true);

    thread::sleep(time::Duration::from_secs(1));
}
