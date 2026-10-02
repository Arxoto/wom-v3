use std::sync::atomic::{AtomicBool, Ordering};

static RUNNING_STAT: AtomicBool = AtomicBool::new(true);

pub fn is_running() -> bool {
    RUNNING_STAT.load(Ordering::SeqCst)
}

pub fn stop_running() {
    RUNNING_STAT.store(false, Ordering::SeqCst);
}
