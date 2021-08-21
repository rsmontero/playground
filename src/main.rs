mod line_buffer;
mod messages;
mod streamer;

use crate::{
    line_buffer::LineBuffer,
    messages::{APIMessage, APIMessageTypes},
    streamer::Streamer,
};

use std::{thread, time};

fn main() {
    //let buffer = LineBuffer::from_stdin();
    let buffer = LineBuffer::from_path("./test_file").unwrap();

    let mut api_str: Streamer<APIMessage> = Streamer::new(buffer, 2);

    api_str.register_action(APIMessageTypes::Red, |m: APIMessage| {
        let s: String = m.to_string();

        println!("Callback MSG: {}", s);

        thread::sleep(time::Duration::from_secs(2));
    });

    thread::spawn(move || {
        api_str.loop_action();
    })
    .join()
    .unwrap();
}
