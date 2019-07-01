use super::Listener;
use alloc::{boxed::Box, vec::Vec};
use core::sync::atomic::{AtomicBool, Ordering};
use lazy_static::lazy_static;
use spin::Mutex;

static UNCONSUMED_TICK: AtomicBool = AtomicBool::new(false);

// Should only be called by interrupt
pub fn update_tick() {
    UNCONSUMED_TICK.store(true, Ordering::Relaxed);
}

type TimerListener = Box<dyn Listener<Value = ()> + Send>;

pub struct TimerEventDispatcher {
    listeners: Vec<TimerListener>,
}

impl TimerEventDispatcher {
    pub fn poll(&mut self) {
        if UNCONSUMED_TICK.compare_and_swap(true, false, Ordering::Relaxed) {
            for listener in &mut self.listeners {
                listener.recv_polled_val(());
            }
        }
    }

    // Returns a handle (listener address) that can be used to remove the listener
    pub fn add_listener(&mut self, listener: TimerListener) -> u64 {
        let handle = listener.as_ref() as *const (_) as *const u64 as u64;
        self.listeners.push(listener);
        handle
    }

    pub fn remove_listener(&mut self, handle: u64) -> Option<TimerListener> {
        let result = self
            .listeners
            .iter()
            .map(|l| l.as_ref() as *const (_) as *const u64 as u64)
            .enumerate()
            .find(|(_, n)| *n == handle);

        result.map(|(i, _)| self.listeners.remove(i))
    }
}

lazy_static! {
    pub static ref TIMER_EVENT_DISPATCHER: Mutex<TimerEventDispatcher> =
        Mutex::new(TimerEventDispatcher {
            listeners: Vec::new()
        });
}

pub struct TimerPrinter;

impl Listener for TimerPrinter {
    type Value = ();

    fn recv_polled_val(&mut self, _: Self::Value) {
        print!(":)");
    }
}
