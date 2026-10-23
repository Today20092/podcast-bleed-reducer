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
    std::thread::spawn(move || {
        let _finished = Finished;
        work()
    })
}
