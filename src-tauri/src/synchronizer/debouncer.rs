use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};

pub struct Debouncer {
    inner: Arc<(Mutex<State>, Condvar)>,
}

struct State {
    last_call: Instant,
    updated: bool,
    task: Option<Box<dyn FnOnce() + Send>>,
}

impl Debouncer {
    pub fn new(delay: Duration) -> Self {
        let inner = Arc::new((
            Mutex::new(State {
                last_call: Instant::now(),
                updated: false,
                task: None,
            }),
            Condvar::new(),
        ));

        let inner_clone = Arc::clone(&inner);

        thread::spawn(move || {
            let (lock, cvar) = &*inner_clone;
            let mut last_scheduled = Instant::now();

            loop {
                let mut state = lock.lock().unwrap();

                // Wait with timeout
                let result = cvar.wait_timeout(state, delay).unwrap();
                state = result.0;

                // If updated recently, restart timeout
                if state.updated {
                    state.updated = false;
                    last_scheduled = state.last_call;
                    continue;
                }

                // Time passed with no updates — run the stored task
                if last_scheduled == state.last_call {
                    if let Some(task) = state.task.take() {
                        drop(state); // drop lock before executing
                        task();
                    }
                }
            }
        });

        Self { inner }
    }

    pub fn call<F>(&self, task: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state.last_call = Instant::now();
        state.updated = true;
        state.task = Some(Box::new(task));
        cvar.notify_one(); // wake up worker
    }
}
