use std::{
    borrow::Borrow,
    collections::HashMap,
    sync::{Arc, Mutex},
};

use tokio_util::sync::CancellationToken;

use crate::application::{error::Error, utils::event::Event};

pub trait CancellationTokenExt {
    fn error_if_cancelled(&self) -> Result<(), TaskError>;
}

pub struct TaskService<TState, TProgress, TResult> {
    inner: Arc<TaskServiceInner<TState, TProgress, TResult>>,
}

pub struct Task<TState, TProgress, TResult> {
    pub id: String,
    pub state: Arc<TState>,
    pub status: TaskStatus<TProgress, TResult>,
}

pub struct TaskHandle<TState, TProgress, TResult> {
    inner: Arc<TaskHandleInner<TState, TProgress, TResult>>,
}

pub struct TaskController<TState, TProgress, TResult> {
    handle: TaskHandle<TState, TProgress, TResult>,
}

pub enum TaskStatus<TProgress, TResult> {
    Pending,
    Running(Arc<TProgress>),
    Completed(Arc<TResult>),
    Cancelled,
    Failed(Arc<Error>),
}

pub enum TaskError {
    Cancelled,
    Failed(Error),
}

pub struct TaskStartEventArgs;

pub struct TaskUpdateEventArgs<TState, TProgress, TResult> {
    pub state: Arc<TState>,
    pub status: TaskStatus<TProgress, TResult>,
}

struct TaskServiceInner<TState, TProgress, TResult> {
    start_event: Event<TaskStartEventArgs>,
    update_event: Event<TaskUpdateEventArgs<TState, TProgress, TResult>>,
    tasks: Mutex<HashMap<String, TaskHandle<TState, TProgress, TResult>>>,
}

struct TaskHandleInner<TState, TProgress, TResult> {
    cancellation_token: CancellationToken,
    start_event: Event<TaskStartEventArgs>,
    update_event: Event<TaskUpdateEventArgs<TState, TProgress, TResult>>,
    id: String,
    state: Arc<TState>,
    status: Mutex<TaskStatus<TProgress, TResult>>,
}

impl<TState, TProgress, TResult> TaskService<TState, TProgress, TResult> {
    pub fn new() -> Self {
        TaskService {
            inner: Arc::new(TaskServiceInner {
                start_event: Event::new(),
                update_event: Event::new(),
                tasks: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub fn start_event(&self) -> &Event<TaskStartEventArgs> {
        &self.inner.start_event
    }

    pub fn update_event(&self) -> &Event<TaskUpdateEventArgs<TState, TProgress, TResult>> {
        &self.inner.update_event
    }

    pub async fn run<F>(&self, task: Task<TState, TProgress, TResult>, f: F) -> Result<(), Error>
    where
        TState: 'static,
        TProgress: Default + 'static,
        TResult: 'static,
        F: AsyncFnOnce(TaskController<TState, TProgress, TResult>) -> Result<TResult, TaskError>,
    {
        let handle = task.into_handle();

        handle
            .start_event()
            .subscribe(self.inner.start_event.clone());

        handle
            .update_event()
            .subscribe(self.inner.update_event.clone());

        {
            let mut tasks = self.inner.tasks.lock().unwrap();
            tasks.insert(handle.id().to_owned(), handle.clone());
        }

        handle.run(f).await;

        Ok(())
    }

    pub fn list(&self) -> Vec<Task<TState, TProgress, TResult>> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.values().map(Task::from).collect()
    }

    pub fn get(&self, id: &str) -> Option<TaskHandle<TState, TProgress, TResult>> {
        let tasks = self.inner.tasks.lock().unwrap();
        tasks.get(id).cloned()
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
                cancellation_token: CancellationToken::new(),
                start_event: Event::new(),
                update_event: Event::new(),
                id: self.id,
                state: self.state,
                status: Mutex::new(self.status),
            }),
        }
    }
}

impl<TState, TProgress, TResult> TaskHandle<TState, TProgress, TResult> {
    pub fn cancellation_token(&self) -> &CancellationToken {
        &self.inner.cancellation_token
    }

    pub fn start_event(&self) -> &Event<TaskStartEventArgs> {
        &self.inner.start_event
    }

    pub fn update_event(&self) -> &Event<TaskUpdateEventArgs<TState, TProgress, TResult>> {
        &self.inner.update_event
    }

    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn status(&self) -> TaskStatus<TProgress, TResult> {
        let inner = self.inner.status.lock().unwrap();
        inner.clone()
    }

    pub fn set_status(&self, status: TaskStatus<TProgress, TResult>) {
        let mut inner = self.inner.status.lock().unwrap();
        *inner = status;
    }

    pub fn update_status(&self, status: TaskStatus<TProgress, TResult>) {
        self.set_status(status);

        let args = TaskUpdateEventArgs::from(Task::from(self));
        self.inner.update_event.invoke(Arc::new(args));
    }

    pub async fn run<F>(&self, f: F)
    where
        TProgress: Default,
        F: AsyncFnOnce(TaskController<TState, TProgress, TResult>) -> Result<TResult, TaskError>,
    {
        let controller = TaskController::new(self.clone());

        self.set_status(TaskStatus::Running(Arc::new(TProgress::default())));

        let args = TaskStartEventArgs::new();
        self.inner.start_event.invoke(Arc::new(args));

        let status = match f(controller).await {
            Ok(result) => TaskStatus::Completed(Arc::new(result)),
            Err(e) => match e {
                TaskError::Cancelled => TaskStatus::Cancelled,
                TaskError::Failed(e) => TaskStatus::Failed(Arc::new(e)),
            },
        };

        self.update_status(status);
    }

    pub fn cancel(&self) {
        match self.status() {
            TaskStatus::Failed(_) => self.update_status(TaskStatus::Cancelled),
            _ => self.inner.cancellation_token.cancel(),
        }
    }
}

impl<TState, TProgress, TResult> TaskController<TState, TProgress, TResult> {
    pub fn new(handle: TaskHandle<TState, TProgress, TResult>) -> Self {
        Self { handle }
    }

    pub fn cancellation_token(&self) -> &CancellationToken {
        self.handle.cancellation_token()
    }

    pub fn report(&self, progress: TProgress) {
        self.handle
            .update_status(TaskStatus::Running(Arc::new(progress)))
    }
}

impl TaskStartEventArgs {
    pub fn new() -> Self {
        Self
    }
}

impl CancellationTokenExt for CancellationToken {
    fn error_if_cancelled(&self) -> Result<(), TaskError> {
        if self.is_cancelled() {
            Err(TaskError::Cancelled)
        } else {
            Ok(())
        }
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

impl<E> From<E> for TaskError
where
    E: Into<Error>,
{
    fn from(value: E) -> Self {
        TaskError::Failed(Into::into(value))
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

impl<TState, TProgress, TResult> Clone for TaskController<TState, TProgress, TResult> {
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
            Self::Cancelled => Self::Cancelled,
            Self::Failed(e) => Self::Failed(e.clone()),
        }
    }
}
