use std::{
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Condvar, Mutex},
    thread,
};

use crate::messages::Message;

type Callback<T> = dyn Fn(T) + Send + Sync + 'static;

pub struct Streamer<T>
where
    T: Message,
    T::MessageType: Hash + Eq,
{
    actions: HashMap<T::MessageType, Arc<Callback<T>>>,
    threads: Arc<(Mutex<u32>, Condvar)>,
}

impl<T> Streamer<T>
where
    T: Message + Send + ToString + 'static,
    T::MessageType: Hash + Eq,
{
    pub fn new(conc: u32) -> Streamer<T> {
        Streamer {
            actions: HashMap::new(),
            threads: Arc::new((Mutex::new(conc), Condvar::new())),
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
}
