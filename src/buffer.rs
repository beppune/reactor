use std::collections::VecDeque;

pub enum PushResult {
    Ok(usize),
    BufferFull,
}
pub trait Buffer {
    fn len(&self) -> usize;
    fn capacity(&self) -> usize;
    fn available(&self) -> usize {
        self.capacity() - self.len()
    }
    fn push(&mut self, bytes: &[u8]) -> PushResult;
}

impl Buffer for VecDeque<u8> {
    fn push(&mut self, bytes: &[u8]) -> PushResult {
        let avail = self.capacity() - self.len();

        if avail < bytes.len() {
            return PushResult::BufferFull;
        }

        self.extend(bytes);

        PushResult::Ok(bytes.len())
    }

    fn capacity(&self) -> usize {
        self.capacity()
    }

    fn len(&self) -> usize {
        self.len()
    }
}
