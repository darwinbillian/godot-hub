use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    error::Error,
    event::{EventDispatcher, EventRepeater},
};

pub struct TaskService<TState, TResult> {
    inner: Arc<TaskServiceInner<TState, TResult>>,
}

pub struct TaskServiceInner<TState, TResult> {
    update_event: EventRepeater<TaskUpdateEventArgs<TState, TResult>>,
    tasks: Mutex<HashMap<String, TaskHandle<TState, TResult>>>,
}

pub struct Task<TState, TResult> {
    pub id: String,
    pub state: Arc<TState>,
    pub status: TaskStatus<TResult>,
}

pub struct TaskHandle<TState, TResult> {
    inner: Arc<TaskHandleInner<TState, TResult>>,
}

pub struct TaskHandleInner<TState, TResult> {
    update_event: EventDispatcher<TaskUpdateEventArgs<TState, TResult>>,
    id: String,
    state: Arc<TState>,
    status: Mutex<TaskStatus<TResult>>,
}

pub enum TaskStatus<TResult> {
    Pending,
    Running,
    Completed(Arc<TResult>),
    Failed(Arc<Error>),
}

pub struct TaskUpdateEventArgs<TState, TResult> {
    pub state: Arc<TState>,
    pub status: TaskStatus<TResult>,
}

impl<TState, TResult> TaskService<TState, TResult> {
    pub fn new() -> Self {
        TaskService {
            inner: Arc::new(TaskServiceInner {
                update_event: EventRepeater::new(),
                tasks: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn update_event(&self) -> &EventRepeater<TaskUpdateEventArgs<TState, TResult>> {
        &self.inner.update_event
    }

    pub async fn start<F>(&self, task: Task<TState, TResult>, f: F) -> Result<(), Error>
    where
        TState: 'static,
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

    pub fn list(&self) -> Vec<Task<TState, TResult>> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.values().map(Task::from).collect()
    }
}

impl<TState, TResult> Task<TState, TResult> {
    pub fn new(id: &str, state: TState) -> Self {
        Task {
            id: id.to_owned(),
            state: Arc::new(state),
            status: TaskStatus::Pending,
        }
    }

    pub fn from(task: &TaskHandle<TState, TResult>) -> Self {
        Self {
            id: task.inner.id.clone(),
            state: task.inner.state.clone(),
            status: task.inner.status.lock().unwrap().clone(),
        }
    }
}

impl<TState, TResult> TaskHandle<TState, TResult> {
    pub fn from(task: Task<TState, TResult>) -> Self {
        Self {
            inner: Arc::new(TaskHandleInner {
                update_event: EventDispatcher::new(),
                id: task.id,
                state: task.state,
                status: Mutex::new(task.status),
            }),
        }
    }

    pub fn update_event(&self) -> &EventDispatcher<TaskUpdateEventArgs<TState, TResult>> {
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

impl<TState, TResult> TaskUpdateEventArgs<TState, TResult> {
    pub fn from(task: &TaskHandle<TState, TResult>) -> Self {
        Self {
            state: task.inner.state.clone(),
            status: task.inner.status.lock().unwrap().clone(),
        }
    }
}

impl<TState, TResult> Clone for TaskService<TState, TResult> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TState, TResult> Clone for TaskHandle<TState, TResult> {
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
