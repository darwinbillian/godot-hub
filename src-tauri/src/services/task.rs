use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::error::Error;

#[derive(Clone)]
pub struct TaskService {
    inner: Arc<TaskServiceInner>,
}

pub struct TaskServiceInner {
    tasks: Mutex<HashMap<String, Task>>,
}

#[derive(Clone)]
pub struct Task {
    inner: Arc<TaskInner>,
}

pub struct TaskInner {
    id: String,
    version: String,
    flavor: String,
    status: Mutex<TaskStatus>,
}

#[derive(Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed(Arc<Error>),
}

impl TaskService {
    pub fn new() -> Self {
        TaskService {
            inner: Arc::new(TaskServiceInner {
                tasks: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub async fn spawn<F>(&self, task: Task, f: F) -> Result<(), Error>
    where
        F: AsyncFnOnce() -> Result<(), Error>,
    {
        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(task.id(), task.clone());
        }

        task.set_status(TaskStatus::Running);

        match f().await {
            Ok(_) => {
                task.set_status(TaskStatus::Completed);

                {
                    let mut tasks = self.inner.tasks.lock().unwrap();
                    tasks.remove(&task.id());
                }
            }
            Err(e) => task.set_status(TaskStatus::Failed(Arc::new(e))),
        }

        Ok(())
    }

    pub fn list(&self) -> Vec<Task> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.values().cloned().collect()
    }
}

impl Task {
    pub fn new(id: &str, version: &str, flavor: &str) -> Self {
        Task {
            inner: Arc::new(TaskInner {
                id: id.to_owned(),
                version: version.to_owned(),
                flavor: flavor.to_owned(),
                status: Mutex::new(TaskStatus::Pending),
            }),
        }
    }

    pub fn id(&self) -> String {
        self.inner.id.clone()
    }

    pub fn version(&self) -> String {
        self.inner.version.clone()
    }

    pub fn flavor(&self) -> String {
        self.inner.flavor.clone()
    }

    pub fn status(&self) -> TaskStatus {
        let inner = self.inner.status.lock().unwrap();
        inner.clone()
    }

    pub fn set_status(&self, status: TaskStatus) {
        let mut inner = self.inner.status.lock().unwrap();
        *inner = status;
    }
}
