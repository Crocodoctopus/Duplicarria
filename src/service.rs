use crossbeam_channel::{Receiver, Sender};
use std::{thread, thread::JoinHandle};

pub struct Service<I, O> {
    sender: Sender<I>,
    receiver: Receiver<O>,
    handle: JoinHandle<()>,
}

impl<I: Send + 'static, O: Send + 'static> Service<I, O> {
    pub fn new(name: &str, f: impl FnOnce(Receiver<I>, Sender<O>) + Send + 'static) -> Self {
        let (sender, service_receiver) = crossbeam_channel::unbounded();
        let (service_sender, receiver) = crossbeam_channel::unbounded();
        let handle = thread::Builder::new()
            .name(name.to_string())
            .spawn(move || f(service_receiver, service_sender))
            .unwrap();

        Self {
            sender,
            receiver,
            handle,
        }
    }

    pub fn send(&self, i: I) {
        self.sender.send(i);
    }

    pub fn recv(&mut self) -> impl Iterator<Item = O> + '_ {
        self.receiver.try_iter()
    }
}