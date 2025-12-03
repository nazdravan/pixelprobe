use crate::ControlError;
use pp_engine::control::{MAX_LINE, Request, Response, parse_request};
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::time::Duration;

/// Most simultaneous client connections.
const MAX_CONNECTIONS: usize = 8;
/// How long a connection waits for the UI thread to answer.
const REPLY_TIMEOUT: Duration = Duration::from_secs(30);

/// A parsed request waiting for the UI thread, plus the way to answer it.
pub struct Pending {
    pub request: Request,
    reply: Sender<Response>,
}

impl Pending {
    /// Answer the request. Safe to call from any thread; a gone client is ignored.
    pub fn reply(self, result: Result<pp_engine::Value, String>) {
        let id = self.request.id.clone();
        let _ = self.reply.send(Response::from_result(id, result));
    }

    /// Split into the request and a reply handle that can be moved to another thread.
    pub fn into_parts(self) -> (Request, Replier) {
        let id = self.request.id.clone();
        (self.request, Replier { id, tx: self.reply })
    }
}

/// Reply handle detached from its request (e.g. for screenshot callbacks).
pub struct Replier {
    id: pp_engine::Value,
    tx: Sender<Response>,
}

impl Replier {
    pub fn reply(self, result: Result<pp_engine::Value, String>) {
        let _ = self.tx.send(Response::from_result(self.id, result));
    }
}

/// Listens on localhost and queues requests for the UI thread.
pub struct ControlServer {
    rx: Receiver<Pending>,
    addr: SocketAddr,
}

impl ControlServer {
    /// Start listening on `127.0.0.1:port` (0 picks a free port). `wake` is called after a
    /// request is queued so the UI can repaint and drain it.
    pub fn start(port: u16, wake: impl Fn() + Send + Sync + 'static) -> Result<Self, ControlError> {
        let listener = TcpListener::bind(("127.0.0.1", port))
            .map_err(|source| ControlError::Bind { port, source })?;
        let addr = listener
            .local_addr()
            .map_err(|source| ControlError::Bind { port, source })?;
        let (tx, rx) = mpsc::channel();
        let wake: Arc<dyn Fn() + Send + Sync> = Arc::new(wake);
        std::thread::Builder::new()
            .name("pp-control-accept".into())
            .spawn(move || accept_loop(listener, tx, wake))
            .map_err(ControlError::Thread)?;
        Ok(Self { rx, addr })
    }

    pub fn addr(&self) -> SocketAddr {
        self.addr
    }

    /// Take every request queued since the last call. Never blocks.
    pub fn drain(&self) -> Vec<Pending> {
        self.rx.try_iter().collect()
    }
}

fn accept_loop(listener: TcpListener, tx: Sender<Pending>, wake: Arc<dyn Fn() + Send + Sync>) {
    let live = Arc::new(AtomicUsize::new(0));
    for stream in listener.incoming() {
        let Ok(mut stream) = stream else { continue };
        if live.load(Ordering::SeqCst) >= MAX_CONNECTIONS {
            let line = Response::error(pp_engine::Value::Null, "too many connections").to_line();
            let _ = writeln!(stream, "{line}");
            continue;
        }
        live.fetch_add(1, Ordering::SeqCst);
        let (tx, wake, live2) = (tx.clone(), wake.clone(), live.clone());
        let spawned = std::thread::Builder::new()
            .name("pp-control-conn".into())
            .spawn(move || {
                serve(stream, &tx, wake.as_ref());
                live2.fetch_sub(1, Ordering::SeqCst);
            });
        if spawned.is_err() {
            live.fetch_sub(1, Ordering::SeqCst);
        }
    }
}

fn serve(stream: TcpStream, tx: &Sender<Pending>, wake: &(dyn Fn() + Send + Sync)) {
    let Ok(mut out) = stream.try_clone() else {
        return;
    };
    let mut reader = BufReader::new(stream);
    loop {
        let mut buf = Vec::new();
        let limit = u64::try_from(MAX_LINE)
            .unwrap_or(u64::MAX)
            .saturating_add(1);
        match (&mut reader).take(limit).read_until(b'\n', &mut buf) {
            Ok(0) | Err(_) => return,
            Ok(_) => {}
        }
        if buf.last() != Some(&b'\n') && buf.len() > MAX_LINE {
            let line = Response::error(pp_engine::Value::Null, "request line too long").to_line();
            let _ = writeln!(out, "{line}");
            return; // the rest of the line is unread; drop the connection
        }
        let text = String::from_utf8_lossy(&buf);
        if text.trim().is_empty() {
            continue;
        }
        let response = match parse_request(&text) {
            Err(resp) => resp,
            Ok(request) => {
                let id = request.id.clone();
                let (reply_tx, reply_rx) = mpsc::channel();
                if tx
                    .send(Pending {
                        request,
                        reply: reply_tx,
                    })
                    .is_err()
                {
                    return; // app is shutting down
                }
                wake();
                reply_rx
                    .recv_timeout(REPLY_TIMEOUT)
                    .unwrap_or_else(|_| Response::error(id, "timed out waiting for the app"))
            }
        };
        if writeln!(out, "{}", response.to_line()).is_err() {
            return;
        }
    }
}
