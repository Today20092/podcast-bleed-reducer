//! Cancellation requests do not acknowledge worker exit. Close waits for the actual thread.
use std::sync::atomic::{AtomicUsize, Ordering};
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
pub(crate) fn active() -> bool {
    ACTIVE.load(Ordering::Acquire) != 0
}
pub(crate) fn spawn<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> std::thread::JoinHandle<T> {
    ACTIVE.fetch_add(1, Ordering::AcqRel);
    struct Finished;
    impl Drop for Finished {
        fn drop(&mut self) {
            ACTIVE.fetch_sub(1, Ordering::AcqRel);
        }
    }
    start_thread(move || {
        let _finished = Finished;
        work()
    })
    .expect("Could not start background worker")
}

fn start_thread<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> std::io::Result<std::thread::JoinHandle<T>> {
    #[cfg(test)]
    if FAIL_NEXT_SPAWN.with(|fail| fail.replace(false)) {
        return Err(std::io::Error::other("injected thread-start failure"));
    }
    std::thread::Builder::new().spawn(work)
}

#[cfg(test)]
thread_local! {
    static FAIL_NEXT_SPAWN: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

#[cfg(test)]
pub(crate) fn fail_next_spawn() {
    FAIL_NEXT_SPAWN.with(|fail| fail.set(true));
}
