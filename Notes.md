# Deadlocks

Busy waiting is ok because it defers to scheduler.
If you loop and never yield, on one thread, you'll deadlock.

## Channels

When you put something into a channel, producers will wake up all the consumers.

## Signals from outside the house (no async context)

Spawn a thread until there's an update. Waker is passed to one of these threads.
Signal in the blocking thread will be when the function exits.

# Methods are tacked on with `impl`

```
impl AsyncHyperPipe {
    pub fn new() ...
    pub async fn pull(&mut self) -> Vec<u8> ...
    pub fn push() ...
}
```

# Async_task

Is a cool library to learn complex low-level rust programs and advanced rust design.

# Futures-lite

Is well-documented, compared to futures.

# Channels

Provide backpressure, really useful with the concurrent system.
