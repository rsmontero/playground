use std::{
    cmp::Eq,
    collections::HashMap,
    hash::Hash,
    sync::{Arc, Mutex},
    thread,
};

use crate::messages::Message;

type Callback<T> = dyn Fn(T) + Send + Sync + 'static;

pub struct Streamer<T>
where
    T: Message,
    T::MessageType: Hash + Eq,
{
    pub actions: HashMap<T::MessageType, Arc<Callback<T>>>,
    //    concurrency: Mutex<u32>,
}

impl<T> Streamer<T>
where
    T: Message + Send + ToString + 'static,
    T::MessageType: Hash + Eq,
{
    pub fn register<F>(&mut self, action: T::MessageType, cb: F)
    where
        F: Fn(T) + Send + Sync + 'static,
    {
        self.actions.insert(action, Arc::new(cb));
    }

    pub fn do_action(&self, msg: T, _thr: bool) {
        let func: &Arc<Callback<T>> = match self.actions.get(msg.message_type()) {
            Some(f) => f,
            None => return,
        };

        let func = func.clone();

        thread::spawn(move || {
            func(msg);
        })
        .join()
        .unwrap();
    }
}
