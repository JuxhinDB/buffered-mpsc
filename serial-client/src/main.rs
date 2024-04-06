use std::io;
use tokio::{io::AsyncWriteExt, net::TcpSocket};

async fn worker(tx: tokio::sync::mpsc::UnboundedSender<u8>) {
    let mut potato = 0;

    loop {
        // It's okay to truncate
        potato = (potato + 1) % 255;
        tx.send(potato).unwrap();
    }
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();

    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;

    let mut iters = 0;
    let mut workers = 1;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(worker(tx.clone()));

    loop {
        if iters % 1000_000 == 0 && workers < 16 {
            println!("iters: {}, workers: {}", iters, workers);
            tokio::spawn(worker(tx.clone()));
            workers += 1;
        }

        let potato = rx.recv().await.unwrap();
        println!("backlog: {}", rx.len());

        stream.write_all(&[potato]).await.unwrap();

        iters += 1;
    }
}
