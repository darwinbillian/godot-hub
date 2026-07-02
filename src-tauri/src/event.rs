use std::sync::{Arc, Mutex};

pub trait Event<T> {
    fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<T> + Send + Sync + 'static;
}

pub trait EventHandler<T> {
    fn invoke(&self, args: Arc<T>);
}

pub struct EventDispatcher<T> {
    handlers: Mutex<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>,
}

pub struct EventAdapter<T> {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
}

pub struct EventAdapterInner<T> {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
}

pub struct EventRepeater<T> {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
}

pub struct EventRepeaterInner<T> {
    handlers: Arc<Mutex<Vec<Arc<dyn EventHandler<T> + Send + Sync>>>>,
}

impl<T> EventDispatcher<T> {
    pub fn new() -> Self {
        Self {
            handlers: Mutex::new(Vec::new()),
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
}

impl<T> EventAdapter<T> {
    pub fn new<E, U>(event: &E) -> Self
    where
        E: Event<U>,
        T: From<Arc<U>> + 'static,
    {
        let handlers = Arc::new(Mutex::new(Vec::new()));

        event.subscribe(EventAdapterInner {
            handlers: handlers.clone(),
        });

        Self { handlers }
    }

    pub fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<T> + Send + Sync + 'static,
    {
        let mut handlers = self.handlers.lock().unwrap();
        handlers.push(Arc::new(handler));
    }
}

impl<T> EventRepeater<T> {
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

    pub fn repeat<E>(&self, event: &E)
    where
        T: 'static,
        E: Event<T>,
    {
        event.subscribe(EventRepeaterInner {
            handlers: self.handlers.clone(),
        });
    }
}

impl<T> Event<T> for EventDispatcher<T> {
    fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<T> + Send + Sync + 'static,
    {
        self.subscribe(handler);
    }
}

impl<U> Event<U> for EventAdapter<U> {
    fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<U> + Send + Sync + 'static,
    {
        self.subscribe(handler);
    }
}

impl<T, U> EventHandler<T> for EventAdapterInner<U>
where
    U: From<Arc<T>>,
{
    fn invoke(&self, args: Arc<T>) {
        let handlers = self.handlers.lock().unwrap().clone();
        handlers.invoke(Arc::new(U::from(args)));
    }
}

impl<T> Event<T> for EventRepeater<T> {
    fn subscribe<E>(&self, handler: E)
    where
        E: EventHandler<T> + Send + Sync + 'static,
    {
        self.subscribe(handler);
    }
}

impl<T> EventHandler<T> for EventRepeaterInner<T> {
    fn invoke(&self, args: Arc<T>) {
        let handlers = self.handlers.lock().unwrap().clone();
        handlers.invoke(args);
    }
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(Arc<T>),
{
    fn invoke(&self, args: Arc<T>) {
        self(args)
    }
}

impl<T> EventHandler<T> for Vec<Arc<dyn EventHandler<T> + Send + Sync>> {
    fn invoke(&self, args: Arc<T>) {
        for handler in self {
            handler.invoke(args.clone())
        }
    }
}
