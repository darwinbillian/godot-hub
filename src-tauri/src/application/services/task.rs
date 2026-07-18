use std::{
    borrow::Borrow,
    collections::HashMap,
    future::Future,
    sync::{Arc, Mutex},
};

use crate::application::{
    error::Error,
    utils::{
        event::Event,
        sync::{CancellationError, CancellationToken, CancellationTokenExt, PauseToken},
    },
};

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
    Paused(Arc<TProgress>),
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
    pause_token: PauseToken,
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

    pub fn run<F, Fut>(&self, task: Task<TState, TProgress, TResult>, f: F)
    where
        TState: Send + Sync + 'static,
        TProgress: Default + Send + Sync + 'static,
        TResult: Send + Sync + 'static,
        F: FnOnce(TaskController<TState, TProgress, TResult>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<TResult, TaskError>> + Send,
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

        tokio::task::spawn(async move {
            handle.run(f).await;
        });
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
                pause_token: PauseToken::new(),
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

    pub fn pause_token(&self) -> &PauseToken {
        &self.inner.pause_token
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

    pub fn state(&self) -> Arc<TState> {
        self.inner.state.clone()
    }

    pub fn status(&self) -> TaskStatus<TProgress, TResult> {
        let inner = self.inner.status.lock().unwrap();
        inner.clone()
    }

    pub fn cancel(&self) {
        match self.status() {
            TaskStatus::Failed(_) => self.update_status(TaskStatus::Cancelled),
            _ => self.inner.cancellation_token.cancel(),
        }
    }

    pub fn pause(&self) {
        self.inner.pause_token.pause();
    }

    pub fn resume(&self) {
        self.inner.pause_token.resume();
    }

    pub async fn run<F, Fut>(&self, f: F)
    where
        TProgress: Default,
        F: FnOnce(TaskController<TState, TProgress, TResult>) -> Fut,
        Fut: Future<Output = Result<TResult, TaskError>>,
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

    fn set_status(&self, status: TaskStatus<TProgress, TResult>) {
        let mut inner = self.inner.status.lock().unwrap();
        *inner = status;
    }

    fn update_status(&self, status: TaskStatus<TProgress, TResult>) {
        self.set_status(status);

        let args = TaskUpdateEventArgs::new(self.state(), self.status());
        self.inner.update_event.invoke(Arc::new(args));
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

    pub async fn cancelled_or_paused(&self) -> Result<(), TaskError> {
        self.cancellation_token().error_if_cancelled()?;

        if !self.handle.pause_token().is_paused() {
            return Ok(());
        }

        if let TaskStatus::Running(progress) = self.handle.status() {
            self.handle
                .update_status(TaskStatus::Paused(progress.clone()))
        }

        tokio::select! {
            biased;
            _ = self.handle.cancellation_token().cancelled() => {
                return Err(TaskError::Cancelled);
            }
            _ = self.handle.pause_token().paused() => {}
        }

        if let TaskStatus::Paused(progress) = self.handle.status() {
            self.handle
                .update_status(TaskStatus::Running(progress.clone()))
        }

        Ok(())
    }
}

impl TaskStartEventArgs {
    pub fn new() -> Self {
        Self
    }
}

impl<TState, TProgress, TResult> TaskUpdateEventArgs<TState, TProgress, TResult> {
    pub fn new(state: Arc<TState>, status: TaskStatus<TProgress, TResult>) -> Self {
        Self { state, status }
    }
}

impl<TState, TProgress, TResult, T> From<T> for Task<TState, TProgress, TResult>
where
    T: Borrow<TaskHandle<TState, TProgress, TResult>>,
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

impl From<CancellationError> for TaskError {
    fn from(_value: CancellationError) -> Self {
        TaskError::Cancelled
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

impl<TProgress, TResult> Clone for TaskStatus<TProgress, TResult> {
    fn clone(&self) -> Self {
        match self {
            Self::Pending => Self::Pending,
            Self::Running(progress) => Self::Running(progress.clone()),
            Self::Paused(progress) => Self::Paused(progress.clone()),
            Self::Completed(result) => Self::Completed(result.clone()),
            Self::Cancelled => Self::Cancelled,
            Self::Failed(e) => Self::Failed(e.clone()),
        }
    }
}
