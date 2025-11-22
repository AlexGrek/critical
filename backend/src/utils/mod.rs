use std::pin::Pin;

// Type alias for boxed futures to make traits dyn compatible
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
