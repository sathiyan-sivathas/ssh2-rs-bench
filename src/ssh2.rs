use anyhow::Result;
use ssh2::Session;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let mut session = Session::new()?;
    let tcp = TcpStream::connect("localhost:22")?;
    session.set_tcp_stream(tcp);
    session.handshake()?;
    session.userauth_pubkey_file(
        "root",
        Some(&Path::new("/etc/ssh/ssh_host_rsa_key.pub")),
        &Path::new("/etc/ssh/ssh_host_rsa_key"),
        None,
    )?;
    session.set_blocking(false);
    let session = Arc::new(session);

    let handle = thread::spawn({
        let session = session.clone();
        move || -> Result<()> {
            loop {
                let sleep = std::cmp::max(session.keepalive_send()?, 30);
                println!("Send next keepalive in {} seconds", sleep);
                thread::sleep(Duration::from_secs(sleep.into()));
            }
        }
    });

    let _listener = session.channel_forward_listen(30000, Some("localhost"), None);

    handle.join().unwrap()?;

    Ok(())
}
