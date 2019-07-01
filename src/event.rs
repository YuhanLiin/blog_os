pub mod keyboard;
pub mod timer;

pub trait Listener {
    type Value;

    fn recv_polled_val(&mut self, polled_val: Self::Value);
}
