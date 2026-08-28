use crate::error::Result;
use windows::{
    core::{Interface, GUID},
    Win32::{
        Foundation::RPC_E_CHANGED_MODE,
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, CLSCTX_ALL,
            COINIT_APARTMENTTHREADED,
        },
    },
};

pub struct Com {}
impl Com {
    fn initialize(flags: windows::Win32::System::Com::COINIT) -> Result<ComGuard> {
        let hresult = unsafe { CoInitializeEx(None, flags) };
        if hresult.is_err() {
            if hresult == RPC_E_CHANGED_MODE {
                ComGuard { initialized: false };
            }
            return Err(format!("CoInitializeEx failed: {:?}", hresult.message()).into());
        }
        Ok(ComGuard { initialized: true })
    }

    pub fn create_instance<T>(class_id: &GUID) -> Result<T>
    where
        T: Interface,
    {
        unsafe { Ok(CoCreateInstance(class_id, None, CLSCTX_ALL)?) }
    }

    /// Will execute init and drop in a safe way, ensuring that all instances created between init and drop are dropped
    pub fn run_with_context<F, T>(f: F) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        let _guard = Self::initialize(COINIT_APARTMENTTHREADED)?;
        f()
    }

    pub fn task_mem_free(ptr: *mut core::ffi::c_void) {
        unsafe { CoTaskMemFree(Some(ptr)) }
    }
}

struct ComGuard {
    initialized: bool,
}

impl Drop for ComGuard {
    fn drop(&mut self) {
        if self.initialized {
            unsafe { CoUninitialize() };
        }
    }
}

// =============================
// ========COM THREAD========
// =============================

use windows::Win32::{
    Foundation::{LPARAM, WPARAM},
    System::Threading::GetCurrentThreadId,
    UI::WindowsAndMessaging::{
        DispatchMessageW, GetMessageW, PeekMessageW, PostThreadMessageW, TranslateMessage, MSG,
        PM_NOREMOVE, WM_APP, WM_QUIT,
    },
};

use crate::utils::spawn_named_thread;

/// custom message used to wake the COM thread up and make it drain pending commands
const WM_COM_EXECUTE: u32 = WM_APP + 1;

type Command<S> = Box<dyn FnOnce(&mut S) + Send>;

/// A dedicated STA thread with a real Win32 message pump, that owns a piece of state `S`
/// created on (and never leaving) that thread.
///
/// COM objects (and specially the callbacks/events they register) are bound to the
/// apartment/thread that created them. Creating them lazily on whatever thread happens to
/// touch them first (e.g. from a plain `LazyLock`) leaves them owned by a thread we don't
/// fully control: it may never pump a message loop, may be a short-lived worker, or may
/// panic, silently breaking event delivery for the rest of the app's life. `ComThread`
/// instead runs forever, so any COM object stored in its state (and any callback it fires)
/// stays valid and responsive for as long as the app is running.
///
/// Typical usage is one `ComThread<S>` per module, held behind a `static LazyLock`:
///
/// ```ignore
/// struct DevicesState {
///     enumerator: IMMDeviceEnumerator,
///     notification_client: IMMNotificationClient,
/// }
///
/// static COM: LazyLock<ComThread<DevicesState>> = LazyLock::new(|| {
///     ComThread::spawn("Devices COM", || unsafe {
///         let enumerator: IMMDeviceEnumerator = Com::create_instance(&MMDeviceEnumerator)?;
///         let notification_client = DevicesManagerEvents.into();
///         enumerator.RegisterEndpointNotificationCallback(&notification_client)?;
///         // state belongs exclusively to this thread from here on
///         Ok(DevicesState { enumerator, notification_client })
///     })
///     .expect("failed to start devices COM thread")
/// });
///
/// let collection = COM.call(|state| unsafe {
///     Ok(state.enumerator.EnumAudioEndpoints(eAll, DEVICE_STATE_ACTIVE)?)
/// })?;
/// ```
pub struct ComThread<S> {
    thread_id: u32,
    sender: crossbeam_channel::Sender<Command<S>>,
    thread: Option<std::thread::JoinHandle<()>>,
}

impl<S> ComThread<S> {
    /// Spawns the COM thread, runs `init` on it to build its state, then starts pumping
    /// messages. Blocks the calling thread until `init` has finished.
    ///
    /// `name` identifies the thread (shows up in debuggers/logs); since `ComThread` is meant
    /// to be reused across modules, give each instance its own distinct name.
    pub fn spawn<F>(name: &str, init: F) -> Result<Self>
    where
        F: FnOnce() -> Result<S> + Send + 'static,
        S: 'static,
    {
        let (sender, receiver) = crossbeam_channel::unbounded::<Command<S>>();
        let (ready_tx, ready_rx) = crossbeam_channel::bounded(1);

        let thread = spawn_named_thread(name, move || {
            let thread_id = unsafe { GetCurrentThreadId() };

            // a thread's message queue is created lazily on its first message-related
            // call; force it to exist now so `PostThreadMessageW` can't be called by a
            // caller (right after `spawn` returns) before the queue exists, which would
            // fail with "invalid thread id"
            unsafe {
                let mut msg = MSG::default();
                let _ = PeekMessageW(&mut msg, None, 0, 0, PM_NOREMOVE);
            }

            // held for the whole thread lifetime: dropping it (which calls
            // `CoUninitialize`) must happen after the message loop returns, not right
            // after `init` runs
            let _guard = match Com::initialize(COINIT_APARTMENTTHREADED) {
                Ok(guard) => guard,
                Err(err) => {
                    ready_tx.send(Err(err)).ok();
                    return;
                }
            };

            let mut state = match init() {
                Ok(state) => state,
                Err(err) => {
                    ready_tx.send(Err(err)).ok();
                    return;
                }
            };

            ready_tx.send(Ok(thread_id)).ok();
            Self::message_loop(&mut state, &receiver);
        });

        let thread_id = ready_rx
            .recv()
            .map_err(|_| "COM thread failed to start")??;

        Ok(Self {
            thread_id,
            sender,
            thread: Some(thread),
        })
    }

    fn message_loop(state: &mut S, receiver: &crossbeam_channel::Receiver<Command<S>>) {
        let mut msg = MSG::default();
        while unsafe { GetMessageW(&mut msg, None, 0, 0) }.into() {
            if msg.message == WM_COM_EXECUTE {
                while let Ok(command) = receiver.try_recv() {
                    // a panicking command must not take the whole COM thread down with
                    // it: every other pending/future `call()` would then block on
                    // `rx.recv()` forever, since nothing would ever be left to answer
                    // them. We accept that `state` may be left in a partially-updated
                    // shape by the unwind.
                    let result =
                        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| command(state)));
                    if let Err(payload) = result {
                        let cause = payload
                            .downcast_ref::<&str>()
                            .map(|s| s.to_string())
                            .or_else(|| payload.downcast_ref::<String>().cloned())
                            .unwrap_or_else(|| "<cause unknown>".to_string());
                        log::error!("COM thread command panicked: {cause}");
                    }
                }
                continue;
            }
            unsafe {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }

    /// Queues `f` to run on the COM thread, with mutable access to its state, without
    /// waiting for it to finish.
    pub fn execute<F>(&self, f: F) -> Result<()>
    where
        F: FnOnce(&mut S) + Send + 'static,
    {
        self.sender
            .send(Box::new(f))
            .map_err(|_| "COM thread is not running")?;
        unsafe { PostThreadMessageW(self.thread_id, WM_COM_EXECUTE, WPARAM(0), LPARAM(0))? };
        Ok(())
    }

    /// Runs `f` on the COM thread, with mutable access to its state, and blocks the caller
    /// until it finishes, returning its result.
    pub fn call<F, T>(&self, f: F) -> Result<T>
    where
        F: FnOnce(&mut S) -> Result<T> + Send + 'static,
        T: Send + 'static,
    {
        let (tx, rx) = crossbeam_channel::bounded(1);
        self.execute(move |state| {
            tx.send(f(state)).ok();
        })?;
        rx.recv()
            .map_err(|_| "COM thread dropped the response channel")?
    }
}

impl<S> Drop for ComThread<S> {
    fn drop(&mut self) {
        unsafe {
            let _ = PostThreadMessageW(self.thread_id, WM_QUIT, WPARAM(0), LPARAM(0));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
