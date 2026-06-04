
pub trait Buffer {

    fn view(&self) -> &[u8];

    fn consume(&mut self, n: usize);

    fn append(&mut self, bytes: &[u8]);

    fn len(&self) -> usize;

    fn available(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Buffer for Vec<u8> {
    fn view(&self) -> &[u8] {
        self
    }

    fn consume(&mut self, n: usize) {
        assert!(n <= self.len(), "consume beyond buffer");
        let n = n.min(self.len());
        self.drain(..n);
    }

    fn append(&mut self, bytes: &[u8]) {
        self.extend(bytes);
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn available(&self) -> usize {
        self.capacity() - self.len()
    }
}
