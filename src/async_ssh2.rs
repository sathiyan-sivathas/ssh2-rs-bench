use anyhow::Result;
use async_ssh2::Session;
use smol::{Async, Timer};
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

fn main() -> Result<()> {
    smol::block_on(async { run().await })
}

async fn run() -> Result<()> {
    let mut session = Session::new()?;
    let tcp = Async::<TcpStream>::connect("localhost:22").await?;

    session.set_tcp_stream(tcp)?;
    session.handshake().await?;
    session
        .userauth_pubkey_file(
            "root",
            Some(&Path::new("/etc/ssh/ssh_host_rsa_key.pub")),
            &Path::new("/etc/ssh/ssh_host_rsa_key"),
            None,
        )
        .await?;
    let session = Arc::new(session);

    let task = smol::Task::spawn({
        let session = session.clone();
        send_keepalives(session)
    });

    let (mut listener, _) = session
        .channel_forward_listen(30000, Some("localhost"), None)
        .await?;

    let _channel = listener.accept().await?;

    task.await?;

    Ok(())
}

async fn send_keepalives(session: Arc<Session>) -> Result<()> {
    loop {
        let sleep = std::cmp::max(session.keepalive_send().await?, 30);
        println!("Send next keepalive in {} seconds", sleep);
        Timer::after(Duration::from_secs(sleep.into())).await;
    }
}
