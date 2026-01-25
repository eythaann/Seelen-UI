use std::pin::Pin;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{self, Duration};

pub fn debounce<F, T>(closure: F, delay: Duration) -> Debounce<T>
where
    F: Fn(T) + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let sender = Arc::new(Mutex::new(sender));
    let debounce_config = Arc::new(Mutex::new(DebounceConfig {
        closure: Box::pin(closure),
        delay,
    }));

    let dup_debounce_config = debounce_config.clone();
    let debounce = Debounce {
        sender: Some(sender),
        thread: Some(std::thread::spawn(move || {
            let debounce_config = dup_debounce_config;
            let mut current_param = None; // The last parameter saved for execution
            let mut last_call_time = None; // The last time a call was received

            loop {
                let result = if current_param.is_none() {
                    receiver
                        .recv()
                        .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
                } else {
                    receiver.recv_timeout(debounce_config.lock().unwrap().delay)
                };

                let now = time::Instant::now();
                match result {
                    Ok(Some(param)) => {
                        // New call received, save parameter and update last call time
                        current_param = Some(param);
                        last_call_time = Some(now);
                    }
                    Ok(None) => {
                        // Terminate signal - execute pending param if any
                        if let Some(param) = current_param.take() {
                            let config = debounce_config.lock().unwrap();
                            (*config.closure)(param);
                        }
                        last_call_time = None;
                    }
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        // Timeout occurred - check if enough time has passed since last call
                        if let Some(param) = current_param.take() {
                            let config = debounce_config.lock().unwrap();
                            let should_execute = last_call_time.is_none()
                                || now.duration_since(last_call_time.unwrap()) >= config.delay;

                            if should_execute {
                                (*config.closure)(param);
                                last_call_time = None;
                            } else {
                                // Not enough time has passed, keep waiting
                                current_param = Some(param);
                            }
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })),
        debounce_config,
    };
    debounce
}

struct DebounceConfig<T> {
    closure: Pin<Box<dyn Fn(T) + Send + Sync + 'static>>,
    delay: Duration,
}

#[cfg(debug_assertions)]
impl<T> Drop for DebounceConfig<T> {
    fn drop(&mut self) {
        println!("drop DebounceConfig {:?}", format!("{:p}", self));
    }
}

#[allow(dead_code)]
pub struct Debounce<T> {
    sender: Option<Arc<Mutex<mpsc::Sender<Option<T>>>>>,
    thread: Option<std::thread::JoinHandle<()>>,
    debounce_config: Arc<Mutex<DebounceConfig<T>>>,
}

impl<T> Debounce<T> {
    pub fn call(&self, param: T) {
        self.sender
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .send(Some(param))
            .unwrap();
    }
    pub fn terminate(&self) {
        self.sender
            .as_ref()
            .unwrap()
            .lock()
            .unwrap()
            .send(None)
            .unwrap();
    }
}
impl<T> Drop for Debounce<T> {
    fn drop(&mut self) {
        self.terminate();
        #[cfg(debug_assertions)]
        println!("drop Debounce {:?}", format!("{:p}", self));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let effect_run_times = Arc::new(Mutex::new(0));
        let param = Arc::new(Mutex::new(0));
        let dup_effect_run_times = effect_run_times.clone();
        let dup_param = param.clone();
        let debounce_fn = debounce(
            move |param| {
                *dup_effect_run_times.lock().unwrap() += 1;
                *dup_param.lock().unwrap() = param;
            },
            std::time::Duration::from_millis(100),
        );
        {
            // Multiple rapid calls - only the last one should execute after delay
            debounce_fn.call(1);
            debounce_fn.call(2);
            debounce_fn.call(3);
            std::thread::sleep(std::time::Duration::from_millis(150));
            assert_eq!(*effect_run_times.lock().unwrap(), 1); // Execute only once
            assert_eq!(*param.lock().unwrap(), 3); // With the last parameter
        }

        {
            // Single call after waiting
            debounce_fn.call(4);
            std::thread::sleep(std::time::Duration::from_millis(150));
            assert_eq!(*effect_run_times.lock().unwrap(), 2);
            assert_eq!(*param.lock().unwrap(), 4);
        }

        {
            // Multiple calls, then terminate before execution
            debounce_fn.call(5);
            debounce_fn.call(6);
            std::thread::sleep(std::time::Duration::from_millis(50)); // Wait less than delay
            debounce_fn.terminate(); // Terminate during debounce period
            std::thread::sleep(std::time::Duration::from_millis(100));
            assert_eq!(*effect_run_times.lock().unwrap(), 3); // Should execute on terminate
            assert_eq!(*param.lock().unwrap(), 6); // With the last parameter
        }
    }

    #[test]
    fn debounce_resets_on_new_calls() {
        let effect_run_times = Arc::new(Mutex::new(0));
        let param = Arc::new(Mutex::new(0));
        let dup_effect_run_times = effect_run_times.clone();
        let dup_param = param.clone();
        let debounce_fn = debounce(
            move |param| {
                *dup_effect_run_times.lock().unwrap() += 1;
                *dup_param.lock().unwrap() = param;
            },
            std::time::Duration::from_millis(100),
        );

        // Call every 50ms for 250ms total - debounce should keep resetting
        debounce_fn.call(1);
        std::thread::sleep(std::time::Duration::from_millis(50));
        debounce_fn.call(2);
        std::thread::sleep(std::time::Duration::from_millis(50));
        debounce_fn.call(3);
        std::thread::sleep(std::time::Duration::from_millis(50));
        debounce_fn.call(4);

        // No execution yet because debounce keeps resetting
        assert_eq!(*effect_run_times.lock().unwrap(), 0);

        // Wait for debounce to complete
        std::thread::sleep(std::time::Duration::from_millis(150));
        assert_eq!(*effect_run_times.lock().unwrap(), 1);
        assert_eq!(*param.lock().unwrap(), 4); // Last parameter
    }
}
