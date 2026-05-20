use std::{
    collections::VecDeque,
    io::{self, ErrorKind},
};

struct Buffer {
    inner: VecDeque<u8>,
}

impl Buffer {
    pub fn new(cap:usize) -> Self {
        Self { inner: VecDeque::with_capacity(cap) }
    }

}

pub trait Framer {
    fn push(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn next_frame(&mut self) -> Option<Vec<u8>>;
}

impl Framer for Buffer {

    fn next_frame(&mut self) -> Option<Vec<u8>> {
        // Servono almeno 2 byte per contenere `\r\n`.
        if self.inner.len() < 2 {
            return None;
        }

        // Scansione lineare alla ricerca di `\r\n`.
        // Il range è `0..len()-1` perché confrontiamo sempre `buffer[i]` con `buffer[i+1]`.
        for i in 0..self.inner.len() - 1 {
            if self.inner[i] == b'\r' && self.inner[i + 1] == b'\n' {
                // Payload = [0..i), delimitatore = 2 byte.
                // Non consumiamo qui: restituiamo anche quanti byte consumare dal buffer.
                let frame: Vec<u8> = self.inner.iter().take(i).copied().collect();
                let to_consume = i + 2; // payload + "\r\n"
                self.inner.drain(0..to_consume);
                return Some(frame);
            }
        }

        None
    }

    fn push(&mut self, bytes: &[u8]) -> io::Result<()> {
        let cap = self.inner.capacity();

        // Caso speciale: i dati nuovi sono più grandi dell'intera capacità.
        // fallisci con errore: non si possono scrivere piu' bytes della capacity
        if bytes.len() > cap {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "data exceeds buffer capacity",
            ));
        }

        // Caso normale: calcoliamo quanto spazio libero c'è.
        let avail = cap - self.inner.len();

        // Se i dati nuovi non ci stanno nello spazio libero,
        // rimuoviamo esattamente l'eccedenza dal fronte (i più vecchi).
        if bytes.len() > avail {
            let diff = bytes.len() - avail;
            self.inner.drain(0..diff);
        }

        // Inserimento: `extend` chiama internamente `push_back` per ogni byte.
        self.inner.extend(bytes);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn new_framer(cap: usize) -> Buffer {
        // Adatta questo costruttore al tuo nuovo tipo
        Buffer::new(cap)
    }

    // =========================
    // BASIC BEHAVIOR
    // =========================

    #[test]
    fn no_data() {
        let mut f = new_framer(20);

        assert!(f.next_frame().is_none());
    }

    #[test]
    fn incomplete_frame() {
        let mut f = new_framer(20);

        f.push(b"hello").unwrap();

        assert!(f.next_frame().is_none());
    }

    #[test]
    fn single_frame() {
        let mut f = new_framer(20);

        f.push(b"hello\r\n").unwrap();

        let frame = f.next_frame().unwrap();
        assert_eq!(frame, b"hello");

        assert!(f.next_frame().is_none());
    }

    #[test]
    fn multiple_frames() {
        let mut f = new_framer(30);

        f.push(b"foo\r\nbar\r\n").unwrap();

        assert_eq!(f.next_frame().unwrap(), b"foo");
        assert_eq!(f.next_frame().unwrap(), b"bar");

        assert!(f.next_frame().is_none());
    }

    // =========================
    // STREAMING BEHAVIOR
    // =========================

    #[test]
    fn split_across_pushes() {
        let mut f = new_framer(20);

        f.push(b"hel").unwrap();
        assert!(f.next_frame().is_none());

        f.push(b"lo\r\n").unwrap();

        assert_eq!(f.next_frame().unwrap(), b"hello");
    }

    #[test]
    fn delimiter_split_across_pushes() {
        let mut f = new_framer(20);

        f.push(b"hello\r").unwrap();
        assert!(f.next_frame().is_none());

        f.push(b"\n").unwrap();

        assert_eq!(f.next_frame().unwrap(), b"hello");
    }

    // =========================
    // EDGE CASES
    // =========================

    #[test]
    fn empty_frame() {
        let mut f = new_framer(10);

        f.push(b"\r\n").unwrap();

        let frame = f.next_frame().unwrap();
        assert!(frame.is_empty());
    }

    #[test]
    fn consecutive_delimiters() {
        let mut f = new_framer(20);

        f.push(b"a\r\n\r\nb\r\n").unwrap();

        assert_eq!(f.next_frame().unwrap(), b"a");
        assert_eq!(f.next_frame().unwrap(), b"");
        assert_eq!(f.next_frame().unwrap(), b"b");

        assert!(f.next_frame().is_none());
    }

    #[test]
    fn dangling_cr() {
        let mut f = new_framer(20);

        f.push(b"hello\rworld").unwrap();

        assert!(f.next_frame().is_none());
    }

    // =========================
    // BUFFER BEHAVIOR
    // =========================

    #[test]
    fn overflow_discards_old_data() {
        let mut f = new_framer(10);

        f.push(b"123456789").unwrap();
        f.push(b"ABCD").unwrap(); // overflow

        // Non testiamo contenuto preciso
        // ma che non crasha e resta consistente
        let _ = f.next_frame();
    }

    #[test]
    fn push_too_large_fails() {
        let mut f = new_framer(5);

        let res = f.push(b"123456");
        assert!(res.is_err());
    }

    #[test]
    fn frame_exactly_fits_capacity() {
        let mut f = new_framer(7);

        f.push(b"hello\r\n").unwrap();

        assert_eq!(f.next_frame().unwrap(), b"hello");
        assert!(f.next_frame().is_none());
    }

    #[test]
    fn partial_frame_lost_due_to_overflow() {
        let mut f = new_framer(10);

        f.push(b"hello12345").unwrap();
        f.push(b"67890\r\n").unwrap();

        let _ = f.next_frame();
    }
}
