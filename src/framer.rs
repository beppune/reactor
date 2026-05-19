use std::{
    collections::VecDeque,
    io::{self, ErrorKind},
};

pub trait Framer {
    fn push(&mut self, bytes: &[u8]) -> io::Result<()>;
    fn next_token(&mut self) -> Option<Vec<u8>>;
}

impl Framer for VecDeque<u8> {

    fn next_token(&mut self) -> Option<Vec<u8>> {
        // Servono almeno 2 byte per contenere `\r\n`.
        if self.len() < 2 {
            return None;
        }

        // Scansione lineare alla ricerca di `\r\n`.
        // Il range è `0..len()-1` perché confrontiamo sempre `buffer[i]` con `buffer[i+1]`.
        for i in 0..self.len() - 1 {
            if self[i] == b'\r' && self[i + 1] == b'\n' {
                // Payload = [0..i), delimitatore = 2 byte.
                // Non consumiamo qui: restituiamo anche quanti byte consumare dal buffer.
                let frame: Vec<u8> = self.iter().take(i).copied().collect();
                let to_consume = i + 2; // payload + "\r\n"
                self.drain(0..to_consume);
                return Some(frame);
            }
        }

        None
    }

    fn push(&mut self, bytes: &[u8]) -> io::Result<()> {
        let cap = self.capacity();

        // Caso speciale: i dati nuovi sono più grandi dell'intera capacità.
        // fallisci con errore: non si possono scrivere piu' bytes della capacity
        if bytes.len() > cap {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "data exceeds buffer capacity",
            ));
        }

        // Caso normale: calcoliamo quanto spazio libero c'è.
        let avail = cap - self.len();

        // Se i dati nuovi non ci stanno nello spazio libero,
        // rimuoviamo esattamente l'eccedenza dal fronte (i più vecchi).
        if bytes.len() > avail {
            let diff = bytes.len() - avail;
            self.drain(0..diff);
        }

        // Inserimento: `extend` chiama internamente `push_back` per ogni byte.
        self.extend(bytes);
        Ok(())
    }
}

