use crate::streamer::Streamer;
use crate::line_buffer::LineBuffer;
use crate::messages::DriverMessage;
use std::{
    cmp::Eq,
    hash::Hash,
    sync::{Arc, Mutex},
};

use std::{thread, thread::JoinHandle};

use strum_macros::{Display, EnumString};

#[derive(EnumString, Display, Debug, Eq, PartialEq, Hash)]
pub enum APIMessageTypes {
    Red,
    Blue,
    Yellow,
}

pub type APIMessage = DriverMessage<APIMessageTypes, true, false>;

////////////////////////////////////////////////////////////////////////////////
pub struct APIManagerProtocol
{
    counter: Mutex<u32>,
}

impl APIManagerProtocol {

    fn new() -> Self {
        Self {
            counter: Mutex::new(0),
        }
    }

    fn red_cb(&self, msg: APIMessage) {
        use std::{time};

        let s: String = msg.to_string();

        {
            println!("[red] atr: {}", self.counter.lock().unwrap());
            println!("[red] msg: {}", s);
        }

        thread::sleep(time::Duration::from_secs(2));
    }

    fn blue_cb(&self, msg: APIMessage) {
        use std::{time};

        let s: String = msg.to_string();
        let mut _counter: u32 = 0;

        {
            let mut c = self.counter.lock().unwrap();
            *c += 1;

            _counter = *c;
        }

        println!("[blue] atr: {}", _counter);
        println!("[blue] msg: {}", s);

        thread::sleep(time::Duration::from_secs(2));
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct APIManager
{
    proto: Arc<APIManagerProtocol>,

    streamer: Arc<Streamer<APIMessage>>,
}

impl APIManager
{
    pub fn new() -> APIManager {

        let buffer = LineBuffer::from_stdin();

        let mut api_str: Streamer<APIMessage> = Streamer::new(buffer, 2);

        let proto = Arc::new(APIManagerProtocol::new());

        let this = proto.clone();

        api_str.register_action(APIMessageTypes::Red,
            move |m: APIMessage| {
                this.red_cb(m);
        });

        let this = proto.clone();

        api_str.register_action(APIMessageTypes::Blue,
            move |m: APIMessage| {
                this.blue_cb(m);
        });

        APIManager {
            proto: proto,
            streamer: Arc::new(api_str),
        }
    }

    pub fn start(&self) -> JoinHandle<()> {
        let _streamer = self.streamer.clone();

        thread::spawn(move || {
            _streamer.loop_action();
        })
    }
}


