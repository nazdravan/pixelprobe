//! End-to-end over a real socket: server thread + a fake UI loop.
#![allow(clippy::unwrap_used, clippy::indexing_slicing)] // test code

use pp_control::{ControlServer, handle};
use pp_engine::Engine;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

fn start() -> (std::net::SocketAddr, Arc<AtomicBool>) {
    let (addr_tx, addr_rx) = std::sync::mpsc::channel();
    let stop = Arc::new(AtomicBool::new(false));
    let stop2 = stop.clone();
    std::thread::spawn(move || {
        let server = ControlServer::start(0, || {}).unwrap();
        addr_tx.send(server.addr()).unwrap();
        let engine = Mutex::new(Engine::new());
        let ctx = egui::Context::default();
        while !stop2.load(Ordering::SeqCst) {
            for p in server.drain() {
                let mut e = engine.lock().unwrap_or_else(|e| e.into_inner());
                handle(&mut e, &ctx, p);
            }
            std::thread::sleep(Duration::from_millis(5));
        }
    });
    (addr_rx.recv().unwrap(), stop)
}

fn ask(stream: &mut TcpStream, reader: &mut BufReader<TcpStream>, line: &str) -> serde_json::Value {
    writeln!(stream, "{line}").unwrap();
    let mut out = String::new();
    reader.read_line(&mut out).unwrap();
    serde_json::from_str(&out).unwrap()
}

#[test]
fn commands_state_and_errors_over_tcp() {
    let (addr, stop) = start();
    let mut stream = TcpStream::connect(addr).unwrap();
    let mut reader = BufReader::new(stream.try_clone().unwrap());

    let list = ask(
        &mut stream,
        &mut reader,
        r#"{"id":1,"method":"app.commands"}"#,
    );
    assert_eq!(list["id"], 1);
    let ids: Vec<&str> = list["ok"]
        .as_array()
        .unwrap()
        .iter()
        .map(|c| c["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"edit.undo"));

    let state = ask(&mut stream, &mut reader, r#"{"id":2,"method":"app.state"}"#);
    assert_eq!(state["ok"]["screen"], "main");

    let disabled = ask(&mut stream, &mut reader, r#"{"id":3,"method":"edit.undo"}"#);
    assert!(disabled["error"].as_str().unwrap().contains("disabled"));

    let unknown = ask(&mut stream, &mut reader, r#"{"id":4,"method":"nope"}"#);
    assert!(
        unknown["error"]
            .as_str()
            .unwrap()
            .contains("unknown command")
    );

    let garbage = ask(&mut stream, &mut reader, "{{{");
    assert!(
        garbage["error"]
            .as_str()
            .unwrap()
            .starts_with("bad request")
    );

    let bad_shot = ask(
        &mut stream,
        &mut reader,
        r#"{"id":5,"method":"ui.screenshot","params":{"path":"x.jpg"}}"#,
    );
    assert!(bad_shot["error"].as_str().unwrap().contains(".png"));

    stop.store(true, Ordering::SeqCst);
}
