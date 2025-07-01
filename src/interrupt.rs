use std::sync::Arc;
use tokio::sync::Mutex;

pub struct InterruptState {
    pub interrupted: bool,
    pub confirmed_abort: bool,
}

impl InterruptState {
    pub fn new() -> Self {
        Self {
            interrupted: false,
            confirmed_abort: false,
        }
    }
}

impl Default for InterruptState {
    fn default() -> Self {
        Self::new()
    }
}

pub fn setup_interrupt_handler(state: Arc<Mutex<InterruptState>>) {
    tokio::spawn(async move {
        loop {
            tokio::signal::ctrl_c()
                .await
                .expect("Failed to listen for Ctrl+C");
            let mut guard = state.lock().await;
            if !guard.interrupted {
                guard.interrupted = true;
                eprintln!("\nInterrupt received. Press Ctrl+C again to abort flashing.");
            } else {
                guard.confirmed_abort = true;
                eprintln!("Aborting.");
                std::process::exit(1);
            }
        }
    });
}
