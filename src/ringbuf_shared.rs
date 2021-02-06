use ringbuf::RingBuffer;
use std::sync::{Arc, Mutex};

pub struct RingBuf<T> {
    prod: Arc<Mutex<ringbuf::Producer<T>>>,
    cons: Arc<Mutex<ringbuf::Consumer<T>>>,
}

impl<T> RingBuf<T> {
    pub fn new(capacity: usize) -> Self {
        let buf = RingBuffer::<T>::new(capacity);
        let (prod, cons) = buf.split();
        Self {
            prod: Arc::new(Mutex::new(prod)),
            cons: Arc::new(Mutex::new(cons)),
        }
    }

    pub fn clone(&self) -> Self {
        Self {
            prod: self.prod.clone(),
            cons: self.cons.clone(),
        }
    }

    pub fn push(&self, item: T) {
        if let Ok(ref mut prod) = self.prod.try_lock() {
            prod.push(item).is_ok();
        }
    }

    pub fn pop(&self) -> Option<T> {
        if let Ok(ref mut cons) = self.cons.try_lock() {
            cons.pop()
        } else {
            None
        }
    }
}
