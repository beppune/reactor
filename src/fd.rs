use crate::buffer::Buffer;

pub struct FdReader<B: Buffer> {
    pub buffer:B,
}
