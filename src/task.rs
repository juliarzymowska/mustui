use std::future::Future;
use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Task<T> {
    sender: Sender<T>,
}

impl<T: Send + 'static> Task<T> {
    pub fn new(sender: Sender<T>) -> Self {
        Self { sender }
    }

    pub fn spawn(&self, future: impl Future<Output = T> + Send + 'static) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let msg = smol::block_on(async_compat::Compat::new(future));
            let _ = sender.send(msg);
        });
    }
}
