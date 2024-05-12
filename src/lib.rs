use std::{
    io,
    sync::{Arc, Mutex},
};
use tokio::net::TcpSocket;
use tokio::net::TcpStream;
use tokio::{
    io::AsyncWriteExt,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

pub async fn mutex_worker(buf: Arc<Mutex<Vec<u8>>>, samples: usize) {
    let mut potato = 0;

    for _ in 0..samples {
        potato = (potato + 1) % 255;

        let mut buf = buf.lock().unwrap();
        buf.push(potato);
        drop(buf);
    }
}

#[allow(clippy::await_holding_lock)]
pub async fn mutex_actor(buf: Arc<Mutex<Vec<u8>>>, samples: usize) -> io::Result<()> {
    // NOTE(jdb): We only allocate two buffers, whenever the consumer
    // consumes, it swaps the worker buffer with the current buffer
    // that is empty, and the current buffer is then consumed/cleared.
    let mut buffer = Vec::with_capacity(samples);
    let mut stream = stream().await?;

    loop {
        let mut current = buf.lock().unwrap();

        // Swap the buffer with an empty one.
        std::mem::swap(&mut buffer, &mut *current);
        drop(current);

        // Consume the buffer
        stream.write_all(&buffer).await?;
    }
}

pub async fn serial_worker(tx: UnboundedSender<u8>, samples: usize) {
    let mut potato = 0;

    for _ in 0..samples {
        potato = (potato + 1) % 255;

        if let Err(e) = tx.send(potato) {
            eprintln!("tx err: {:?}", e.0);
            break;
        }
    }
}

pub async fn serial_actor(mut rx: UnboundedReceiver<u8>) -> io::Result<()> {
    let mut stream = stream().await?;

    loop {
        match rx.recv().await {
            Some(msg) => stream.write_all(&[msg]).await?,
            None => eprintln!("received none from channel"),
        };
    }
}

async fn stream() -> io::Result<TcpStream> {
    let addr = "127.0.0.1:8080".parse().unwrap();
    let socket = TcpSocket::new_v4()?;
    socket.connect(addr).await
}
