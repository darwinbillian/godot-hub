use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{error::Error, event::EventHandler, services::install::Installation};

#[derive(Clone)]
pub struct TaskService {
    inner: Arc<TaskServiceInner>,
}

pub struct TaskServiceInner {
    tasks: Mutex<HashMap<String, TaskHandle>>,
    update_event: TaskUpdateEvent,
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
    update_event: TaskUpdateEvent,
}

#[derive(Clone)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed(Arc<Installation>),
    Failed(Arc<Error>),
}

pub struct TaskUpdateEvent {
    handlers: Mutex<Vec<Arc<dyn EventHandler<TaskUpdateEventArgs> + Send + Sync>>>,
}

#[derive(Clone)]
pub struct TaskUpdateEventArgs {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: TaskStatus,
}

impl TaskService {
    pub fn new() -> Self {
        TaskService {
            inner: Arc::new(TaskServiceInner {
                tasks: Mutex::new(HashMap::new()),
                update_event: TaskUpdateEvent::new(),
            }),
        }
    }

    pub fn update_event(&self) -> &TaskUpdateEvent {
        &self.inner.update_event
    }

    pub async fn start<F>(&self, task: Task, f: F) -> Result<(), Error>
    where
        F: AsyncFnOnce() -> Result<Installation, Error>,
    {
        let id = task.id.clone();
        let handle = TaskHandle::from(task);
        let inner = self.inner.clone();

        handle.update_event().subscribe(move |args| {
            inner.update_event.invoke(args);
        });

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(id.clone(), handle.clone());
        }

        handle.set_status(TaskStatus::Running);

        match f().await {
            Ok(installation) => {
                handle.set_status(TaskStatus::Completed(Arc::new(installation)));

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
                update_event: TaskUpdateEvent::new(),
            }),
        }
    }

    pub fn set_status(&self, status: TaskStatus) {
        {
            let mut inner = self.inner.status.lock().unwrap();
            *inner = status;
        }

        let args = TaskUpdateEventArgs::from(self);
        self.inner.update_event.invoke(args);
    }

    pub fn update_event(&self) -> &TaskUpdateEvent {
        &self.inner.update_event
    }
}

impl TaskUpdateEvent {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
        }
    }

    pub fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<TaskUpdateEventArgs> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(Arc::new(handler))
    }

    pub fn invoke(&self, args: TaskUpdateEventArgs) {
        let handlers = self.handlers.lock().unwrap().clone();
        handlers.invoke(args);
    }
}

impl TaskUpdateEventArgs {
    pub fn from(task: &TaskHandle) -> Self {
        Self {
            id: task.inner.id.clone(),
            version: task.inner.version.clone(),
            flavor: task.inner.flavor.clone(),
            status: task.inner.status.lock().unwrap().clone(),
        }
    }
}
