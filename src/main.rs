mod messages;
mod streamer;

use crate::messages::{APIMessage, APIMessageTypes, Message};

use crate::streamer::Streamer;

use std::{thread, time};

fn main() {
    let msg = String::from("Red 124 SUCCESS aGVsbG8gd29ybGQ=");

    let mut api_str: Streamer<APIMessage> = Streamer::new(2);

    api_str.register_action(APIMessageTypes::Red, |m: APIMessage| {
        let s: String = m.to_string();

        println!("Callback MSG: {}", s);

        thread::sleep(time::Duration::from_secs(2));
    });

    let msg1 = APIMessage::parse(&msg).unwrap();
    api_str.do_action(msg1, true);
    let msg1 = APIMessage::parse(&msg).unwrap();
    api_str.do_action(msg1, true);
    let msg1 = APIMessage::parse(&msg).unwrap();
    api_str.do_action(msg1, true);

    let msg2 = String::from("Yellow 124 SUCCESS aGVsbG8gd29ybGQ=");

    api_str.do_action(APIMessage::parse(&msg2).unwrap(), true);

    thread::sleep(time::Duration::from_secs(3));
}
