use std::io;
use tokio::{
    io::AsyncWriteExt,
    net::TcpSocket,
    task::JoinSet,
    time::Duration,
};

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

pub async fn runner(samples: u64) -> io::Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();

    let socket = TcpSocket::new_v4()?;
    let mut stream = socket.connect(addr).await?;

    let mut iters = 0;
    let mut workers = 1;

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let mut join_set = JoinSet::new();

    join_set.spawn(worker(tx.clone()));

    let mut buffer = Vec::with_capacity(64);

    while iters < samples {
        println!("backlog: {}", rx.len());

        if iters % 1000 == 0 && workers < 64 {
            println!("iters: {}, workers: {}", iters, workers);
            join_set.spawn(worker(tx.clone()));
            workers += 1;
        }

        if let Some(potato) = rx.recv().await {
            buffer.push(potato);

            if buffer.len() == buffer.capacity() {
                // Hold the connection for 50ms to simulate a workload.
                tokio::time::sleep(Duration::from_millis(50)).await;

                stream.write_all(&buffer).await.unwrap();
                buffer.clear();
            }
        }

        iters += 1;
    }

    Ok(())

}

#[tokio::main]
async fn main() -> io::Result<()> {
    runner(10_000_000).await
}
