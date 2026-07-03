use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    error::Error,
    event::{EventDispatcher, EventRepeater},
};

pub struct TaskService<TState, TProgress, TResult> {
    inner: Arc<TaskServiceInner<TState, TProgress, TResult>>,
}

pub struct TaskServiceInner<TState, TProgress, TResult> {
    update_event: EventRepeater<TaskUpdateEventArgs<TState, TProgress, TResult>>,
    tasks: Mutex<HashMap<String, TaskHandle<TState, TProgress, TResult>>>,
}

pub struct Task<TState, TProgress, TResult> {
    pub id: String,
    pub state: Arc<TState>,
    pub status: TaskStatus<TProgress, TResult>,
}

pub struct TaskHandle<TState, TProgress, TResult> {
    inner: Arc<TaskHandleInner<TState, TProgress, TResult>>,
}

pub struct TaskReporter<TState, TProgress, TResult> {
    handle: TaskHandle<TState, TProgress, TResult>,
}

pub struct TaskHandleInner<TState, TProgress, TResult> {
    update_event: EventDispatcher<TaskUpdateEventArgs<TState, TProgress, TResult>>,
    id: String,
    state: Arc<TState>,
    status: Mutex<TaskStatus<TProgress, TResult>>,
}

pub enum TaskStatus<TProgress, TResult> {
    Pending,
    Running(Arc<TProgress>),
    Completed(Arc<TResult>),
    Failed(Arc<Error>),
}

pub struct TaskUpdateEventArgs<TState, TProgress, TResult> {
    pub state: Arc<TState>,
    pub status: TaskStatus<TProgress, TResult>,
}

impl<TState, TProgress, TResult> TaskService<TState, TProgress, TResult> {
    pub fn new() -> Self {
        TaskService {
            inner: Arc::new(TaskServiceInner {
                update_event: EventRepeater::new(),
                tasks: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn update_event(&self) -> &EventRepeater<TaskUpdateEventArgs<TState, TProgress, TResult>> {
        &self.inner.update_event
    }

    pub async fn run<F>(&self, task: Task<TState, TProgress, TResult>, f: F) -> Result<(), Error>
    where
        TState: 'static,
        TProgress: Default + 'static,
        TResult: Send + Sync + 'static,
        F: AsyncFnOnce(TaskReporter<TState, TProgress, TResult>) -> Result<TResult, Error>,
    {
        let id = task.id.clone();
        let handle = task.into_handle();

        self.inner.update_event.repeat(handle.update_event());

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(id.clone(), handle.clone());
        }

        handle.set_status(TaskStatus::Running(Arc::new(TProgress::default())));

        let reporter = TaskReporter::new(handle.clone());
        let result = match f(reporter).await {
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

    pub fn list(&self) -> Vec<Task<TState, TProgress, TResult>> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.values().map(Task::from).collect()
    }
}

impl<TState, TProgress, TResult> Task<TState, TProgress, TResult> {
    pub fn new(id: &str, state: TState) -> Self {
        Task {
            id: id.to_owned(),
            state: Arc::new(state),
            status: TaskStatus::Pending,
        }
    }

    pub fn into_handle(self) -> TaskHandle<TState, TProgress, TResult> {
        TaskHandle {
            inner: Arc::new(TaskHandleInner {
                update_event: EventDispatcher::new(),
                id: self.id,
                state: self.state,
                status: Mutex::new(self.status),
            }),
        }
    }
}

impl<TState, TProgress, TResult> TaskHandle<TState, TProgress, TResult> {
    pub fn update_event(
        &self,
    ) -> &EventDispatcher<TaskUpdateEventArgs<TState, TProgress, TResult>> {
        &self.inner.update_event
    }

    pub fn set_status(&self, status: TaskStatus<TProgress, TResult>) {
        {
            let mut inner = self.inner.status.lock().unwrap();
            *inner = status;
        }

        let args = TaskUpdateEventArgs::from(Task::from(self));
        self.inner.update_event.invoke(Arc::new(args));
    }
}

impl<TState, TProgress, TResult> TaskReporter<TState, TProgress, TResult> {
    pub fn new(handle: TaskHandle<TState, TProgress, TResult>) -> Self {
        Self { handle }
    }

    pub fn report(&self, progress: TProgress) {
        self.handle
            .set_status(TaskStatus::Running(Arc::new(progress)))
    }
}

impl<TState, TProgress, TStatus, T> From<T> for Task<TState, TProgress, TStatus>
where
    T: Borrow<TaskHandle<TState, TProgress, TStatus>>,
{
    fn from(value: T) -> Self {
        let value = value.borrow();
        Self {
            id: value.inner.id.clone(),
            state: value.inner.state.clone(),
            status: value.inner.status.lock().unwrap().clone(),
        }
    }
}

impl<TState, TProgress, TResult> From<Task<TState, TProgress, TResult>>
    for TaskUpdateEventArgs<TState, TProgress, TResult>
{
    fn from(value: Task<TState, TProgress, TResult>) -> Self {
        Self {
            state: value.state,
            status: value.status,
        }
    }
}

impl<TState, TProgress, TResult> Clone for TaskService<TState, TProgress, TResult> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TState, TProgress, TResult> Clone for TaskHandle<TState, TProgress, TResult> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}

impl<TState, TProgress, TResult> Clone for TaskReporter<TState, TProgress, TResult> {
    fn clone(&self) -> Self {
        Self {
            handle: self.handle.clone(),
        }
    }
}

impl<TProgress, TResult> Clone for TaskStatus<TProgress, TResult> {
    fn clone(&self) -> Self {
        match self {
            Self::Pending => Self::Pending,
            Self::Running(progress) => Self::Running(progress.clone()),
            Self::Completed(result) => Self::Completed(result.clone()),
            Self::Failed(e) => Self::Failed(e.clone()),
        }
    }
}
