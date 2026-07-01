use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    error::Error,
    event::{EventDispatcher, EventRepeater},
};

pub struct TaskService<TResult> {
    inner: Arc<TaskServiceInner<TResult>>,
}

pub struct TaskServiceInner<TResult> {
    update_event: EventRepeater<TaskUpdateEventArgs<TResult>>,
    tasks: Mutex<HashMap<String, TaskHandle<TResult>>>,
}

pub struct Task<TResult> {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: TaskStatus<TResult>,
}

pub struct TaskHandle<TResult> {
    inner: Arc<TaskHandleInner<TResult>>,
}

pub struct TaskHandleInner<TResult> {
    update_event: EventDispatcher<TaskUpdateEventArgs<TResult>>,
    id: String,
    version: String,
    flavor: String,
    status: Mutex<TaskStatus<TResult>>,
}

pub enum TaskStatus<TResult> {
    Pending,
    Running,
    Completed(Arc<TResult>),
    Failed(Arc<Error>),
}

pub struct TaskUpdateEventArgs<TResult> {
    pub id: String,
    pub version: String,
    pub flavor: String,
    pub status: TaskStatus<TResult>,
}

impl<TResult> TaskService<TResult> {
    pub fn new() -> Self {
        TaskService {
            inner: Arc::new(TaskServiceInner {
                update_event: EventRepeater::new(),
                tasks: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn update_event(&self) -> &EventRepeater<TaskUpdateEventArgs<TResult>> {
        &self.inner.update_event
    }

    pub async fn start<F>(&self, task: Task<TResult>, f: F) -> Result<(), Error>
    where
        TResult: Send + Sync + 'static,
        F: AsyncFnOnce() -> Result<TResult, Error>,
    {
        let id = task.id.clone();
        let handle = TaskHandle::from(task);

        self.inner.update_event.repeat(handle.update_event());

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(id.clone(), handle.clone());
        }

        handle.set_status(TaskStatus::Running);

        let result = match f().await {
            Ok(result) => result,
            Err(e) => {
                handle.set_status(TaskStatus::Failed(Arc::new(e)));
                return Ok(());
            }
        };

        handle.set_status(TaskStatus::Completed(Arc::new(result)));

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.remove(&id);
        }

        Ok(())
    }

    pub fn list(&self) -> Vec<Task<TResult>> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.values().map(Task::from).collect()
    }
}

impl<TResult> Task<TResult> {
    pub fn new(id: &str, version: &str, flavor: &str) -> Self {
        Task {
            id: id.to_owned(),
            version: version.to_owned(),
            flavor: flavor.to_owned(),
            status: TaskStatus::Pending,
        }
    }

    pub fn from(task: &TaskHandle<TResult>) -> Self {
        Self {
            id: task.inner.id.clone(),
            version: task.inner.version.clone(),
            flavor: task.inner.flavor.clone(),
            status: task.inner.status.lock().unwrap().clone(),
        }
    }
}

impl<TResult> TaskHandle<TResult> {
    pub fn from(task: Task<TResult>) -> Self {
        Self {
            inner: Arc::new(TaskHandleInner {
                update_event: EventDispatcher::new(),
                id: task.id,
                version: task.version,
                flavor: task.flavor,
                status: Mutex::new(task.status),
            }),
        }
    }

    pub fn update_event(&self) -> &EventDispatcher<TaskUpdateEventArgs<TResult>> {
        &self.inner.update_event
    }

    pub fn set_status(&self, status: TaskStatus<TResult>) {
        {
            let mut inner = self.inner.status.lock().unwrap();
            *inner = status;
        }

        let args = TaskUpdateEventArgs::from(self);
        self.inner.update_event.invoke(Arc::new(args));
    }
}

impl<TResult> TaskUpdateEventArgs<TResult> {
    pub fn from(task: &TaskHandle<TResult>) -> Self {
        Self {
            id: task.inner.id.clone(),
            version: task.inner.version.clone(),
            flavor: task.inner.flavor.clone(),
            status: task.inner.status.lock().unwrap().clone(),
        }
    }
}

impl<TResult> Clone for TaskService<TResult> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TResult> Clone for TaskHandle<TResult> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TResult> Clone for TaskStatus<TResult> {
    fn clone(&self) -> Self {
        match self {
            Self::Pending => Self::Pending,
            Self::Running => Self::Running,
            Self::Completed(result) => Self::Completed(result.clone()),
            Self::Failed(e) => Self::Failed(e.clone()),
        }
    }
}
