use std::pin::Pin;
use std::sync::{mpsc, Arc, Mutex};
use std::time::{self, Duration};

pub fn throttle<F, T>(closure: F, delay: Duration) -> Throttle<T>
where
    F: Fn(T) + Send + Sync + 'static,
    T: Send + Sync + 'static,
{
    let (sender, receiver) = mpsc::channel();
    let sender = Arc::new(Mutex::new(sender));
    let throttle_config = Arc::new(Mutex::new(ThrottleConfig {
        closure: Box::pin(closure),
        delay,
    }));

    let dup_throttle_config = throttle_config.clone();
    let throttle = Throttle {
        sender: Some(sender),
        thread: Some(std::thread::spawn(move || {
            let throttle_config = dup_throttle_config;
            let mut current_param = None; // The last parameter saved for execution
            let mut last_execution_time = None; // The last time the closure was executed

            loop {
                let result = if current_param.is_none() {
                    receiver
                        .recv()
                        .map_err(|_| mpsc::RecvTimeoutError::Disconnected)
                } else {
                    receiver.recv_timeout(throttle_config.lock().unwrap().delay)
                };

                let now = time::Instant::now();
                match result {
                    Ok(Some(param)) => {
                        let config = throttle_config.lock().unwrap();
                        let should_execute = last_execution_time.is_none()
                            || now.duration_since(last_execution_time.unwrap()) >= config.delay;

                        if should_execute {
                            (*config.closure)(param);
                            current_param = None;
                            last_execution_time = Some(now);
                        } else {
                            current_param = Some(param);
                        }
                    }
                    Ok(None) => current_param = None, // Terminate signal
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        if let Some(param) = current_param.take() {
                            let config = throttle_config.lock().unwrap();
                            (*config.closure)(param);
                            // Note: Timeout execution does not update last_execution_time
                            // This allows immediate execution on next call if enough time has passed
                        }
                    }
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                }
            }
        })),
        throttle_config,
    };
    throttle
}

struct ThrottleConfig<T> {
    closure: Pin<Box<dyn Fn(T) + Send + Sync + 'static>>,
    delay: Duration,
}

#[cfg(debug_assertions)]
impl<T> Drop for ThrottleConfig<T> {
    fn drop(&mut self) {
        println!("drop ThrottleConfig {:?}", format!("{:p}", self));
    }
}

#[allow(dead_code)]
pub struct Throttle<T> {
    sender: Option<Arc<Mutex<mpsc::Sender<Option<T>>>>>,
    thread: Option<std::thread::JoinHandle<()>>,
    throttle_config: Arc<Mutex<ThrottleConfig<T>>>,
}

impl<T> Throttle<T> {
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
impl<T> Drop for Throttle<T> {
    fn drop(&mut self) {
        self.terminate();
        #[cfg(debug_assertions)]
        println!("drop Throttle {:?}", format!("{:p}", self));
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
        let throttle_fn = throttle(
            move |param| {
                *dup_effect_run_times.lock().unwrap() += 1;
                *dup_param.lock().unwrap() = param;
            },
            std::time::Duration::from_millis(100),
        );
        {
            throttle_fn.call(1);
            throttle_fn.call(2);
            throttle_fn.call(3);
            std::thread::sleep(std::time::Duration::from_millis(200));
            assert_eq!(*effect_run_times.lock().unwrap(), 2); // Execute the last parameter after delay
            assert_eq!(*param.lock().unwrap(), 3);
        }

        {
            throttle_fn.call(4);
            std::thread::sleep(std::time::Duration::from_millis(200));
            assert_eq!(*effect_run_times.lock().unwrap(), 3);
            assert_eq!(*param.lock().unwrap(), 4);
        }

        {
            throttle_fn.call(5);
            throttle_fn.call(6);
            throttle_fn.terminate(); // Terminate before last execution
            std::thread::sleep(std::time::Duration::from_millis(200));
            assert_eq!(*effect_run_times.lock().unwrap(), 4);
            assert_eq!(*param.lock().unwrap(), 5);
        }
    }
}
