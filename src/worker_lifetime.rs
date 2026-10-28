//! Cancellation requests do not acknowledge worker exit. Close waits for the actual thread.
use std::sync::atomic::{AtomicUsize, Ordering};
static ACTIVE: AtomicUsize = AtomicUsize::new(0);
pub(crate) fn active() -> bool {
    ACTIVE.load(Ordering::Acquire) != 0
}
pub(crate) fn spawn<T: Send + 'static>(
    work: impl FnOnce() -> T + Send + 'static,
) -> std::io::Result<std::thread::JoinHandle<T>> {
    ACTIVE.fetch_add(1, Ordering::AcqRel);
    struct Finished;
    impl Drop for Finished {
        fn drop(&mut self) {
            ACTIVE.fetch_sub(1, Ordering::AcqRel);
        }
    }
    let finished = Finished;
    spawn_untracked(move || {
        let _finished = finished;
        work()
    })
}

// Persistent playback has its own cancel/join owner and must not block ordinary close.
pub(crate) fn spawn_untracked<T: Send + 'static>(
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, atomic::AtomicBool, mpsc};

    struct Released(Arc<AtomicBool>);
    impl Drop for Released {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }

    #[test]
    fn worker_start_failure_releases_captured_work_without_running_it() {
        let released = Arc::new(AtomicBool::new(false));
        let owned = Released(Arc::clone(&released));
        let (sender, receiver) = mpsc::channel();
        fail_next_spawn();
        let result = spawn(move || {
            let _owned = owned;
            let _ = sender.send(());
        });
        assert!(result.is_err());
        assert!(released.load(Ordering::Acquire));
        assert!(receiver.try_recv().is_err());
    }

    #[test]
    fn registered_workers_hold_owned_work_until_completion_or_unwinding() {
        for panic in [false, true] {
            let released = Arc::new(AtomicBool::new(false));
            let owned = Released(Arc::clone(&released));
            let (ready, started) = mpsc::channel();
            let (finish, finished) = mpsc::channel();
            let worker = spawn(move || {
                let _owned = owned;
                ready.send(()).unwrap();
                finished.recv().unwrap();
                assert!(!panic, "injected worker panic");
            })
            .unwrap();
            started.recv().unwrap();
            assert!(active(), "close must wait while the worker still owns work");
            assert!(!released.load(Ordering::Acquire));
            finish.send(()).unwrap();
            assert_eq!(worker.join().is_err(), panic);
            assert!(released.load(Ordering::Acquire));
        }
    }
}
