use std::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Task<T> {
    sender: Sender<T>,
}

impl<T: Send + 'static> Task<T> {
    pub fn new(sender: Sender<T>) -> Self {
        Self { sender }
    }

    pub fn spawn(&self, f: impl FnOnce() -> T + Send + 'static) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let _ = sender.send(f());
        });
    }
}
