use super::Listener;
use alloc::{boxed::Box, vec::Vec};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1};
use spin::Mutex;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
}

// This lock is accessed by keyboard interrupt, so any other contention should
// be minimized and done with interrupts disabled
static SCANCODE: Mutex<Option<u8>> = Mutex::new(None);

pub fn update_scancode(scancode: u8) {
    *SCANCODE.lock() = Some(scancode);
}

type KeyboardListener = Box<dyn Listener<Value = DecodedKey> + Send>;

pub struct KeyboardEventDispatcher {
    listeners: Vec<KeyboardListener>,
}

impl KeyboardEventDispatcher {
    pub fn poll(&mut self) {
        let mut scancode = None;
        x86_64::instructions::interrupts::without_interrupts(|| {
            scancode = SCANCODE.lock().take();
        });

        self.poll_key(scancode);
    }

    pub fn poll_key(&mut self, mut scancode: Option<u8>) {
        let mut keyboard = KEYBOARD.lock();

        if let Some(scancode) = scancode.take() {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    for listener in &mut self.listeners {
                        listener.recv_polled_val(key);
                    }
                }
            }
        }
    }

    // Returns a handle (listener address) that can be used to remove the listener
    pub fn add_listener(&mut self, listener: KeyboardListener) -> u64 {
        let handle = listener.as_ref() as *const (_) as *const u64 as u64;
        self.listeners.push(listener);
        handle
    }

    pub fn remove_listener(&mut self, handle: u64) -> Option<KeyboardListener> {
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
    pub static ref KEYBOARD_EVENT_DISPATCHER: Mutex<KeyboardEventDispatcher> =
        Mutex::new(KeyboardEventDispatcher {
            listeners: Vec::new()
        });
}

pub struct KeyPrinter;

impl Listener for KeyPrinter {
    type Value = DecodedKey;

    fn recv_polled_val(&mut self, key: Self::Value) {
        match key {
            DecodedKey::Unicode(ch) => print!("{}", ch),
            DecodedKey::RawKey(key) => print!("{:?}", key),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    struct MockListener {
        expected: DecodedKey,
    }

    impl Listener for MockListener {
        type Value = DecodedKey;

        fn recv_polled_val(&mut self, key: Self::Value) {
            assert_eq!(key, self.expected);
        }
    }

    impl MockListener {
        fn new(expected: DecodedKey) -> Self {
            Self { expected }
        }
    }

    test!(add_remove {
        let mut dispatcher = KeyboardEventDispatcher { listeners: Vec::new() };
        let l1 = Box::new(MockListener::new(DecodedKey::Unicode('c')));
        let l2 = Box::new(MockListener::new(DecodedKey::Unicode('c')));
        let h1 = dispatcher.add_listener(l1);
        assert_eq!(dispatcher.listeners.len(), 1);
        let h2 = dispatcher.add_listener(l2);
        assert_eq!(dispatcher.listeners.len(), 2);
        dispatcher.remove_listener(h2);
        assert_eq!(dispatcher.listeners.len(), 1);
        dispatcher.remove_listener(h1);
        assert_eq!(dispatcher.listeners.len(), 0);
    });

    test!(correct_key {
        let mut dispatcher = KeyboardEventDispatcher { listeners: Vec::new() };
        let mock = Box::new(MockListener::new(DecodedKey::Unicode(' ')));
        dispatcher.add_listener(mock);
        // Should do nothing
        dispatcher.poll_key(None);
        // Should check off the flag
        dispatcher.poll_key(Some(57));
    });
}
