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
    tasks: Mutex<HashMap<String, TaskHandle>>,
}

pub struct Task {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: TaskStatus,
}

#[derive(Clone)]
pub struct TaskHandle {
    inner: Arc<TaskHandleInner>,
}

pub struct TaskHandleInner {
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

    pub async fn start<F>(&self, task: Task, f: F) -> Result<(), Error>
    where
        F: AsyncFnOnce() -> Result<(), Error>,
    {
        let id = task.id.clone();
        let handle = TaskHandle::from(task);

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(id.clone(), handle.clone());
        }

        handle.set_status(TaskStatus::Running);

        match f().await {
            Ok(_) => {
                handle.set_status(TaskStatus::Completed);

                let mut tasks = self.inner.tasks.lock().unwrap();
                tasks.remove(&id);
            }
            Err(e) => handle.set_status(TaskStatus::Failed(Arc::new(e))),
        }

        Ok(())
    }

    pub fn list(&self) -> Vec<Task> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.values().map(Task::from).collect()
    }
}

impl Task {
    pub fn new(id: &str, version: &str, flavor: &str) -> Self {
        Task {
            id: id.to_owned(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
            status: TaskStatus::Pending,
        }
    }

    pub fn from(task: &TaskHandle) -> Self {
        Self {
            id: task.inner.id.clone(),
            version: task.inner.version.clone(),
            flavor: task.inner.flavor.clone(),
            status: task.inner.status.lock().unwrap().clone(),
        }
    }
}

impl TaskHandle {
    pub fn from(task: Task) -> Self {
        Self {
            inner: Arc::new(TaskHandleInner {
                id: task.id,
                version: task.version,
                flavor: task.flavor,
                status: Mutex::new(task.status),
            }),
        }
    }

    pub fn set_status(&self, status: TaskStatus) {
        let mut inner = self.inner.status.lock().unwrap();
        *inner = status;
    }
}
