use std::{
    sync::{mpsc, Arc},
    thread,
};

use parking_lot::Mutex;

pub type TaskResult = Result<String, String>;

#[derive(Clone)]
pub struct TaskSystem {
    completed_tx: mpsc::Sender<TaskResult>,
    completed_rx: Arc<Mutex<mpsc::Receiver<TaskResult>>>,
    completed: Arc<Mutex<Vec<TaskResult>>>,
}

impl TaskSystem {
    pub fn new() -> Self {
        let (completed_tx, completed_rx) = mpsc::channel();
        Self {
            completed_tx,
            completed_rx: Arc::new(Mutex::new(completed_rx)),
            completed: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn spawn<F>(&self, name: impl Into<String>, job: F)
    where
        F: FnOnce() -> TaskResult + Send + 'static,
    {
        let name = name.into();
        let tx = self.completed_tx.clone();
        thread::Builder::new()
            .name(name)
            .spawn(move || {
                let _ = tx.send(job());
            })
            .expect("spawn task");
    }

    pub fn poll_completed(&self) {
        while let Ok(result) = self.completed_rx.lock().try_recv() {
            self.completed.lock().push(result);
        }
    }

    pub fn drain_completed(&self) -> Vec<TaskResult> {
        self.poll_completed();
        std::mem::take(&mut *self.completed.lock())
    }
}

impl Default for TaskSystem {
    fn default() -> Self {
        Self::new()
    }
}
