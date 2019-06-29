use alloc::{boxed::Box, vec::Vec};
use lazy_static::lazy_static;
use pc_keyboard::{layouts, DecodedKey, Keyboard, ScancodeSet1};
use spin::Mutex;

lazy_static! {
    static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
        Mutex::new(Keyboard::new(layouts::Us104Key, ScancodeSet1));
}

lazy_static! {
    static ref SCANCODE: Mutex<Option<u8>> = Mutex::new(None);
}

pub fn update_scancode(scancode: u8) {
    *SCANCODE.lock() = Some(scancode);
}

pub trait Task {
    type Value;

    fn recv_polled_val(&mut self, polled_val: Self::Value);
}

type TaskBox = Box<dyn Task<Value = DecodedKey> + Send>;

pub struct KeyboardTaskRunner {
    tasks: Vec<TaskBox>,
}

impl KeyboardTaskRunner {
    pub fn poll(&mut self) {
        let mut keyboard = KEYBOARD.lock();
        let mut scancode = None;
        x86_64::instructions::interrupts::without_interrupts(|| {
            scancode = SCANCODE.lock().take();
        });

        if let Some(scancode) = scancode.take() {
            if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
                if let Some(key) = keyboard.process_keyevent(key_event) {
                    for task in &mut self.tasks {
                        task.recv_polled_val(key);
                    }
                }
            }
        }
    }

    pub fn add_task(&mut self, task: TaskBox) {
        self.tasks.push(task);
    }
}

lazy_static! {
    pub static ref KEYBOARD_TASK_RUNNER: Mutex<KeyboardTaskRunner> =
        Mutex::new(KeyboardTaskRunner { tasks: Vec::new() });
}

pub struct KeyPrinter;

impl Task for KeyPrinter {
    type Value = DecodedKey;

    fn recv_polled_val(&mut self, key: Self::Value) {
        match key {
            DecodedKey::Unicode(ch) => print!("{}", ch),
            DecodedKey::RawKey(key) => print!("{:?}", key),
        }
    }
}
