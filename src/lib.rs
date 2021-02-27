// #![deny(missing_docs)]
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;

pub enum LoopState {
    Continue,
    Break,
}
pub trait Cancellabel {
    type Error;
    fn for_each(&mut self) -> Result<LoopState, Self::Error>;

    fn run(&mut self) -> Result<(), Self::Error> {
        loop {
            match self.for_each() {
                Ok(LoopState::Continue) => {}
                Ok(LoopState::Break) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    fn spawn(mut self) -> Handle<Self::Error>
    where
        Self: Sized + Send + 'static,
        Self::Error: Send + 'static,
    {
        let keep_running = Arc::new(AtomicBool::new(true));
        let jh = {
            let keep_running = keep_running.clone();
            thread::spawn(move || {
                while keep_running.load(Ordering::SeqCst) {
                    match self.for_each() {
                        Ok(LoopState::Continue) => {}
                        Ok(LoopState::Break) => break,
                        Err(e) => return Err(e),
                    }
                }
                Ok(())
            })
        };
        Handle::new(keep_running, jh)
    }
}

pub struct Handle<E> {
    keep_running: Arc<AtomicBool>,
    executor: thread::JoinHandle<Result<(), E>>,
}
impl<E> Handle<E> {
    pub fn new(keep_running: Arc<AtomicBool>, jh: thread::JoinHandle<Result<(), E>>) -> Self {
        Handle {
            keep_running,
            executor: jh,
        }
    }
    #[warn(non_fmt_panic)]
    pub fn wait(self) -> Result<(), E> {
        match self.executor.join() {
            Ok(r) => r,
            Err(e) => {
                // propagate the panic!
                panic!("{:?}", e)
            }
        }
    }

    pub fn cancle(&self) {
        self.keep_running.store(false, Ordering::SeqCst);
    }

    pub fn canceller(&self) -> Canceller {
        Canceller {
            keep_running: self.keep_running.clone(),
        }
    }
}

#[derive(Clone)]
pub struct Canceller {
    keep_running: Arc<AtomicBool>,
}

impl Canceller {
    pub fn cancle(&self) {
        self.keep_running.store(false, Ordering::SeqCst);
    }
}
