use std::io;
use std::sync::Arc;
use tokio::{
    io::AsyncWriteExt,
    net::{TcpSocket, TcpStream},
    sync::Mutex,
    time::Duration,
};

async fn worker(buf: Arc<Mutex<Vec<u8>>>) {
    let mut potato = 0;

    loop {
        potato = (potato + 1) % 255;

        let mut buf = buf.lock().await;
        buf.push(potato);
        drop(buf);

        tokio::time::sleep(Duration::from_millis(1)).await;
    }
}

async fn actor(buf: Arc<Mutex<Vec<u8>>>, mut stream: TcpStream, samples: u64) {
    let mut iters = 0;
    let mut workers = 1;

    while iters < samples {
        let mut current = buf.lock().await;

        // Swap the buffer with an empty one.
        let mut buffer = Vec::with_capacity(current.capacity());
        std::mem::swap(&mut buffer, &mut *current);
        drop(current);

        // Write the buffer to the socket
        stream.write_all(&buffer).await.unwrap();

        if iters % 1000 == 0 && workers < 64 {
            tokio::spawn(worker(Arc::clone(&buf)));
            workers += 1;
        }

        iters += 1;
    }
}

pub async fn runner(samples: u64) -> io::Result<()> {
    let addr = "127.0.0.1:8080".parse().unwrap();

    let socket = TcpSocket::new_v4()?;
    let stream = socket.connect(addr).await?;

    let buf = std::sync::Arc::new(Mutex::new(Vec::with_capacity(64)));

    tokio::join!(worker(Arc::clone(&buf)), actor(Arc::clone(&buf), stream, samples));

    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    runner(10_000_000).await
}
