use std::sync::{Arc, Mutex};

pub trait EventHandler<T> {
    fn invoke(&self, args: Arc<T>);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(Arc<T>),
{
    fn invoke(&self, args: Arc<T>) {
        self(args)
    }
}

impl<T, E> EventHandler<T> for Vec<E>
where
    E: AsRef<dyn EventHandler<T> + Send + Sync>,
{
    fn invoke(&self, args: Arc<T>) {
        for handler in self {
            handler.as_ref().invoke(args.clone())
        }
    }
}

pub struct Event<T> {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
}

impl<T> Event<T> {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<T> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(Arc::new(handler));
    }

    pub fn invoke(&self, args: Arc<T>) {
        let handlers = self.handlers.lock().unwrap().clone();
        handlers.invoke(args);
    }

    pub fn map<F, U>(&self, f: F) -> Event<U>
    where
        F: Fn(Arc<T>) -> U + Send + Sync + 'static,
        U: 'static,
    {
        let event = Event::<U>::new();

        self.subscribe({
            let event = event.clone();
            move |args| event.invoke(Arc::new(f(args)))
        });

        event
    }

    pub fn filter_map<F, U>(&self, f: F) -> Event<U>
    where
        F: Fn(Arc<T>) -> Option<U> + Send + Sync + 'static,
        U: 'static,
    {
        let event = Event::<U>::new();

        self.subscribe({
            let event = event.clone();
            move |args| {
                if let Some(args) = f(args) {
                    event.invoke(Arc::new(args));
                }
            }
        });

        event
    }
}

impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self {
            handlers: self.handlers.clone(),
        }
    }
}

impl<T> EventHandler<T> for Event<T> {
    fn invoke(&self, args: Arc<T>) {
        self.invoke(args);
    }
}
