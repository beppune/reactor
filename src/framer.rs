use crate::buffer::Buffer;

pub enum Frame {
    Text(String),
}

pub trait Framer {
    fn next_frame<B: Buffer>(&mut self, buffer: &mut B) -> Option<usize>;
}

#[derive(Default,Debug)]
struct LineFramer {
    strip_cr: bool,
}

impl Framer for  LineFramer{

    fn next_frame<B: Buffer>(&mut self, buffer: &mut B) -> Option<usize> {

        if buffer.is_empty() {
            return None;
        }

        let slice = buffer.view();

        for i in 0..slice.len() {
            if slice[i] == b'\n' {
                let to = if self.strip_cr { i } else { i + 1 };
                return Some(to);
            }
        }

        None
    }
}

