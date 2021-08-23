use std::{
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Condvar, Mutex},
    thread,
};

use crate::line_buffer::LineReader;
use crate::messages::Message;

type Callback<T> = dyn Fn(T) + Send + Sync + 'static;

pub struct Streamer<T>
where
    T: Message,
    T::MessageType: Hash + Eq,
{
    actions: HashMap<T::MessageType, Arc<Callback<T>>>,
    threads: Arc<(Mutex<u32>, Condvar)>,

    line_buffer: Mutex<Box<dyn LineReader + Send + Sync + 'static>>,
}

impl<T> Streamer<T>
where
    T: Message + Send + ToString + 'static,
    T::MessageType: Hash + Eq,
{
    pub fn new<LB>(buff: LB, conc: u32) -> Streamer<T>
    where
        LB: LineReader + Send + Sync + 'static,
    {
        Streamer {
            actions: HashMap::new(),
            threads: Arc::new((Mutex::new(conc), Condvar::new())),

            line_buffer: Mutex::new(Box::new(buff)),
        }
    }

    pub fn register_action<F>(&mut self, action: T::MessageType, cb: F)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.actions.insert(action, Arc::new(cb));
    }

    pub fn do_action(&self, msg: T, thr: bool) {
        let func = match self.actions.get(msg.message_type()) {
            Some(f) => f,
            None => return,
        };

        if thr {
            {
                let (mtx, cv) = &*self.threads;
                let mut left_thr = mtx.lock().unwrap();

                while *left_thr == 0 {
                    left_thr = cv.wait(left_thr).unwrap();
                }

                *left_thr -= 1;
            }

            let func = Arc::clone(&func);
            let threads = Arc::clone(&self.threads);

            thread::spawn(move || {
                func(msg);

                let (mtx, cv) = &*threads;

                let mut left_thr = mtx.lock().unwrap();

                *left_thr += 1;

                cv.notify_one();
            });
        } else {
            func(msg);
        }
    }

    pub fn loop_action(&self) {
        let threaded: bool = {
            let (mtx, _) = &*self.threads;
            *(mtx.lock().unwrap()) > 0
        };

        loop {
            let s: String = {
                let mut lb = self.line_buffer.lock().unwrap();

                match lb.read_line() {
                    Some(s) => s,
                    None => continue,
                }
            };

            let msg = match T::parse(&s) {
                Some(m) => m,
                None => continue,
            };

            self.do_action(msg, threaded);
        }
    }
}
