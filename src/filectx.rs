use std::sync::{Arc, Mutex};

/// Il tipo di Task che il tuo executor accetta.
pub type Task = Box<dyn FnOnce() + Send + 'static>;

/// Context di dominio per una lettura stream/chunked.
/// - Contiene configurazione (chunk_size)
/// - Contiene stato condiviso (buffer accumulato)
/// - Contiene le callback registrate (on_chunk, on_eof)
///
/// Non conosce fd/reactor/handler.
/// È pensato per vivere nei Task dell'executor.
#[derive(Clone)]
pub struct FileReadContext {
    inner: Arc<Mutex<Vec<u8>>>,
    chunk_size: usize,

    // Callback ripetibile: viene chiamata molte volte.
    on_chunk: Arc<Mutex<Option<Box<dyn FnMut(Vec<u8>, &FileReadContext) + Send>>>>,

    // Callback one-shot: viene chiamata una sola volta a EOF.
    on_eof: Arc<Mutex<Option<Box<dyn FnOnce(FileReadContext) + Send>>>>,
}

impl FileReadContext {
    /// Crea un nuovo context con la configurazione desiderata.
    pub fn new(chunk_size: usize) -> Self {
        let ctx = Self {
            inner: Arc::new(Mutex::new(Vec::new())),
            chunk_size,
            on_chunk: Arc::new(Mutex::new(None)),
            on_eof: Arc::new(Mutex::new(None)),
        };

        *ctx.on_chunk.lock().unwrap() = Some(Box::new(
                |chunk:Vec<u8>, c:&FileReadContext| {
                    c.push_bytes(&chunk);
                }
        ));

        ctx
    }

    /// Dimensione chunk usata dall'handler I/O per sapere quanto leggere.
    pub fn chunk_size(&self) -> usize {
        self.chunk_size
    }

    /// Registra la callback per ogni chunk.
    /// La callback è FnMut perché può essere chiamata più volte.
    pub fn on_chunk<F>(&self, f: F)
    where
        F: FnMut(Vec<u8>, &FileReadContext) + Send + 'static,
    {
        *self.on_chunk.lock().unwrap() = Some(Box::new(f));
    }

    /// Registra la callback di EOF.
    /// È FnOnce perché EOF accade una sola volta.
    pub fn on_eof<F>(&self, f: F)
    where
        F: FnOnce(FileReadContext) + Send + 'static,
    {
        *self.on_eof.lock().unwrap() = Some(Box::new(f));
    }

    /// Utility di dominio: accumula bytes.
    /// Tipicamente chiamata dentro on_chunk.
    pub fn push_bytes(&self, bytes: &[u8]) {
        let mut guard = self.inner.lock().unwrap();
        guard.extend_from_slice(bytes);
    }

    /// Accesso in sola lettura allo stato accumulato, con lock incapsulato.
    /// Utile per ispezione/logging/parsing senza consumare lo stato.
    pub fn with_bytes<R, F>(&self, f: F) -> R
    where
        F: FnOnce(&[u8]) -> R,
    {
        let guard = self.inner.lock().unwrap();
        f(&guard)
    }

    /// Consuma definitivamente il contenuto accumulato.
    /// Da chiamare tipicamente dentro on_eof (che riceve il context by value).
    ///
    /// Panica se il context è ancora condiviso: indica una violazione del lifecycle
    /// (es. qualcuno ha clonat*o* il context e lo tiene vivo oltre EOF).
    pub fn take(self) -> Vec<u8> {
        Arc::try_unwrap(self.inner)
            .expect("context still shared")
            .into_inner()
            .unwrap()
    }

    /// Crea un Task per un evento "chunk".
    /// Ritorna None se non c'è una callback registrata.

    pub fn make_chunk_task(&self, chunk: Vec<u8>) -> Option<Task> {
        let ctx = self.clone();

        Some(Box::new(move || {
            let mut slot = ctx.on_chunk.lock().unwrap();

            if let Some(cb) = slot.as_mut() {
                (cb)(chunk, &ctx);
            }
            // else: nessuna callback → nessun effetto
        }))
    }


    /// Crea un Task per l'evento EOF.
    /// Consuma la callback FnOnce (quindi può essere chiamato solo una volta).
    pub fn make_eof_task(&self) -> Option<Task> {
        // Estrae la callback FnOnce (take) e la consuma.
        let cb = self.on_eof.lock().unwrap().take()?;
        let ctx = self.clone();

        Some(Box::new(move || {
            cb(ctx);
        }))
    }
}

