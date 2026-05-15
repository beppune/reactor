use std::{
    collections::VecDeque,
    io::{self, ErrorKind},
};

/// Framer: buffer circolare a dimensione fissa con logica di framing basata su delimitatore.
///
/// Utilizza `VecDeque<u8>` come storage sottostante. VecDeque è già internamente
/// un ring buffer (array contiguo con head + len e wrap-around automatico),
/// quindi non serve reimplementare la struttura dati da zero.
///
/// L'unica cosa che VecDeque NON fa è il comportamento "a dimensione fissa":
/// quando è pieno, VecDeque rialloca e cresce. La dimensione fissa viene
/// forzata manualmente nel metodo `push`, drenando i byte più vecchi
/// prima di inserire quelli nuovi.
///
/// Il protocollo di framing è line-based: la sequenza `\r\n` separa un frame
/// dal successivo (come in HTTP/1.1, SMTP, Redis RESP, ecc.).
struct Framer {
    buffer: VecDeque<u8>,
    // Nota: non serve un campo `start` per tracciare la posizione di scansione.
    // La scansione parte sempre dall'indice 0 del buffer (cioè dal byte più vecchio).
    //
    // Se in futuro il buffer diventasse molto grande, si potrebbe reintrodurre
    // `start` come ottimizzazione per evitare di riscansionare byte già esaminati.
    // In quel caso `start` andrebbe impostato a `len - 1` dopo ogni scansione fallita,
    // perché l'ultimo byte potrebbe essere un `\r` il cui `\n` arriva nel prossimo push.
}

impl Framer {
    /// Crea un Framer con capacità fissa `c`.
    ///
    /// `with_capacity` è solo un hint di pre-allocazione: VecDeque non impone
    /// un limite rigido. La dimensione fissa è enforciata dalla logica in `push`.
    fn with_capacity(c: usize) -> Self {
        Self {
            buffer: VecDeque::with_capacity(c),
        }
    }

    /// Tenta di estrarre un frame completo dal buffer.
    ///
    /// Un frame è una sequenza di byte terminata da `\r\n`.
    /// Il delimitatore NON viene incluso nel frame restituito.
    ///
    /// Restituisce:
    /// - `Ok((Vec<u8>, usize))` — (payload, bytes_da_consumare) se trovato
    /// - `Err(WouldBlock)` — se non c'è ancora un frame completo
    ///
    /// Perché `WouldBlock` e non `InvalidData`:
    /// nel contesto di un reactor/event loop, "non c'è ancora un frame"
    /// non è un errore — è la condizione normale di "dati insufficienti,
    /// riprova quando arrivano altri byte". WouldBlock esprime esattamente
    /// questa semantica non-bloccante.
    fn try_frame(&self) -> io::Result<(Vec<u8>, usize)> {
        // Servono almeno 2 byte per contenere `\r\n`.
        if self.buffer.len() < 2 {
            return Err(ErrorKind::WouldBlock.into());
        }

        // Scansione lineare alla ricerca di `\r\n`.
        // Il range è `0..len()-1` perché confrontiamo sempre `buffer[i]` con `buffer[i+1]`.
        for i in 0..self.buffer.len() - 1 {
            if self.buffer[i] == b'\r' && self.buffer[i + 1] == b'\n' {
                // Payload = [0..i), delimitatore = 2 byte.
                // Non consumiamo qui: restituiamo anche quanti byte consumare dal buffer.
                let frame: Vec<u8> = self.buffer.iter().take(i).copied().collect();
                let to_consume = i + 2; // payload + "\r\n"
                return Ok((frame, to_consume));
            }
        }

        Err(ErrorKind::WouldBlock.into())
    }

    /// Consuma i primi `n` byte dal buffer.
    ///
    /// Tipicamente `n` è il secondo elemento della tupla restituita da `try_frame`
    /// (payload + delimitatore).
    fn consume(&mut self, n: usize) {
        self.buffer.drain(0..n);
    }

    /// Inserisce un blocco di byte nel buffer, rispettando la dimensione fissa.
    ///
    /// Se non c'è spazio sufficiente, i byte più vecchi vengono scartati
    /// per fare posto ai nuovi. Questo è il comportamento classico di un
    /// ring buffer: i dati più recenti hanno sempre la priorità.
    ///
    /// ATTENZIONE: se il drain rimuove byte che facevano parte di un frame
    /// non ancora estratto con `try_frame`, quei dati sono persi.
    /// È responsabilità del chiamante estrarre i frame prima che il buffer
    /// si riempia, oppure dimensionare il buffer in modo adeguato.
    fn push(&mut self, bytes: &[u8]) -> io::Result<()> {
        let cap = self.buffer.capacity();

        // Caso speciale: i dati nuovi sono più grandi dell'intera capacità.
        // fallisci con errore: non si possono scrivere piu' bytes della capacity
        if bytes.len() > cap {
            return Err(io::Error::new(
                ErrorKind::InvalidInput,
                "data exceeds buffer capacity",
            ));
        }

        // Caso normale: calcoliamo quanto spazio libero c'è.
        let avail = cap - self.buffer.len();

        // Se i dati nuovi non ci stanno nello spazio libero,
        // rimuoviamo esattamente l'eccedenza dal fronte (i più vecchi).
        if bytes.len() > avail {
            let diff = bytes.len() - avail;
            self.buffer.drain(0..diff);
        }

        // Inserimento: `extend` chiama internamente `push_back` per ogni byte.
        self.buffer.extend(bytes);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    /// Buffer vuoto: nessun frame disponibile.
    #[test]
    fn no_data() {
        let mut framer = Framer::with_capacity(20);
        let frame = framer.try_frame();
        assert!(matches!(frame, Err(e) if e.kind() == ErrorKind::WouldBlock));
    }

    /// Dati presenti ma senza delimitatore: frame incompleto.
    #[test]
    fn incomplete_frame() {
        let mut framer = Framer::with_capacity(20);
        framer.push(b"hello").unwrap();
        assert!(matches!(framer.try_frame(), Err(e) if e.kind() == ErrorKind::WouldBlock));
    }

    /// Un singolo frame completo: viene estratto senza il delimitatore.
    #[test]
    fn single_frame() {
        let mut framer = Framer::with_capacity(20);
        framer.push(b"hello\r\n").unwrap();
        let (frame, n) = framer.try_frame().unwrap();
        assert_eq!(frame, b"hello");
        framer.consume(n);
    }

    /// Due frame nello stesso push: vengono estratti uno alla volta.
    /// Dopo il primo consume, il secondo è ancora nel buffer.
    #[test]
    fn two_frames() {
        let mut framer = Framer::with_capacity(30);
        framer.push(b"foo\r\nbar\r\n").unwrap();

        let (frame, n) = framer.try_frame().unwrap();
        assert_eq!(frame, b"foo");
        framer.consume(n);

        let (frame, n) = framer.try_frame().unwrap();
        assert_eq!(frame, b"bar");
        framer.consume(n);
    }

    /// Frame spezzato su due push successive: funziona anche con read parziali.
    #[test]
    fn split_across_pushes() {
        let mut framer = Framer::with_capacity(20);
        framer.push(b"hel").unwrap();
        framer.push(b"lo\r\n").unwrap();

        let (frame, n) = framer.try_frame().unwrap();
        assert_eq!(frame, b"hello");
        framer.consume(n);
    }
}
