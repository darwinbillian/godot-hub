use std::sync::Arc;

pub trait EventHandler<T> {
    fn invoke(&self, args: T);
}

impl<T, F> EventHandler<T> for F
where
    F: Fn(T),
{
    fn invoke(&self, args: T) {
        self(args)
    }
}

impl<T> EventHandler<T> for Vec<Arc<dyn EventHandler<T> + Send + Sync>>
where
    T: Clone,
{
    fn invoke(&self, args: T) {
        for handler in self {
            handler.invoke(args.clone())
        }
    }
}
