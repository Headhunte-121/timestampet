use rusqlite::Connection;
use std::sync::{mpsc, MutexGuard};
use std::thread;

pub enum DbTask {
    Task(Box<dyn FnOnce(&mut MutexGuard<'static, Connection>) + Send>),
}

pub struct DbTaskQueue {
    high_priority_tx: mpsc::Sender<DbTask>,
    low_priority_tx: mpsc::Sender<DbTask>,
}

impl DbTaskQueue {
    pub fn new() -> Self {
        let (high_priority_tx, high_priority_rx) = mpsc::channel::<DbTask>();
        let (low_priority_tx, low_priority_rx) = mpsc::channel::<DbTask>();

        thread::spawn(move || {
            loop {
                match high_priority_rx.try_recv() {
                    Ok(task) => {
                        Self::execute_task(task);
                        continue;
                    }
                    Err(mpsc::TryRecvError::Disconnected) => break,
                    Err(mpsc::TryRecvError::Empty) => {
                        match low_priority_rx.try_recv() {
                            Ok(task) => {
                                Self::execute_task(task);
                                continue;
                            }
                            Err(mpsc::TryRecvError::Disconnected) => break,
                            Err(mpsc::TryRecvError::Empty) => {
                                thread::sleep(std::time::Duration::from_millis(10));
                            }
                        }
                    }
                }
            }
        });

        Self {
            high_priority_tx,
            low_priority_tx,
        }
    }

    fn execute_task(task: DbTask) {
        let DbTask::Task(func) = task;
        if let Ok(mut conn) = crate::db::get_db_connection() {
            func(&mut conn);
        } else {
            eprintln!("Failed to get db connection in task queue worker");
        }
    }

    pub fn push_high_priority<F>(&self, func: F)
    where
        F: FnOnce(&mut MutexGuard<'static, Connection>) + Send + 'static,
    {
        let _ = self.high_priority_tx.send(DbTask::Task(Box::new(func)));
    }

    pub fn push_low_priority<F>(&self, func: F)
    where
        F: FnOnce(&mut MutexGuard<'static, Connection>) + Send + 'static,
    {
        let _ = self.low_priority_tx.send(DbTask::Task(Box::new(func)));
    }
}
