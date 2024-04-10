use std::{io, time::Duration};
use tokio::{io::AsyncWriteExt, net::TcpSocket, task::JoinSet, time::interval};

async fn worker(tx: tokio::sync::mpsc::UnboundedSender<u8>) {
    let mut potato = 0;

    loop {
        potato = (potato + 1) % 255;

        if let Err(e) = tx.send(potato) {
            eprintln!("failed to send potato: {}", e);
            break;
        }

        tokio::time::sleep(Duration::from_millis(1)).await;
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

    let mut join_set = JoinSet::new();
    join_set.spawn(worker(tx.clone()));

    let mut interval = interval(Duration::from_millis(10_000_000));
    interval.tick().await;

    'outer: loop {
        println!("backlog: {}", rx.len());
        if iters % 1000 == 0 && workers < 64 {
            println!("iters: {}, workers: {}", iters, workers);
            tokio::spawn(worker(tx.clone()));
            workers += 1;
        }

        tokio::select! {
            biased;
            _ = interval.tick() => {
                println!("time elapsed with {} workers and backlog: {}", workers, rx.len());
                join_set.shutdown().await;
                break 'outer;
            },
            potato = rx.recv() => {
                if let Some(potato) = potato {
                    stream.write_all(&[potato]).await.unwrap();
                }
            }
        }

        iters += 1;
    }

    Ok(())
}
