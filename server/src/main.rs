use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

#[tokio::main]
async fn main() -> io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;

    loop {
        let (mut socket, _) = listener.accept().await?;

        // We'll process each socket concurrently.
        tokio::spawn(async move {
            let mut buf = vec![0; 1024];

            // We want to read the data into our buffer, then write
            // it back to the same socket.
            loop {
                let n = socket
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                // Perhaps we don't want to accept empty buffers,
                // but for now we can ignore that.

                socket
                    .write_all(&buf[..n])
                    .await
                    .expect("failed to write data to socket");
            }
        });
    }
}
