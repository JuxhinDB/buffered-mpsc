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
            let mut buf = vec![0; 65535];

            // We want to read the data into our buffer, then write
            // it back to the same socket.
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        println!(
                            "received {} bytes, msg: {}",
                            n,
                            String::from_utf8_lossy(buf.clone()[..n].to_vec().as_slice())
                        );

                        socket
                            .write_all(&buf[..n])
                            .await
                            .expect("failed to write data to socket");
                    }
                    Err(e) => {
                        eprintln!("failed to read from socket; err = {:?}", e);
                        return;
                    }
                }
            }
        });
    }
}
