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
