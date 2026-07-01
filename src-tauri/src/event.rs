use std::sync::Arc;

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

impl<T> EventHandler<T> for Vec<Arc<dyn EventHandler<T> + Send + Sync>> {
    fn invoke(&self, args: Arc<T>) {
        for handler in self {
            handler.invoke(args.clone())
        }
    }
}
