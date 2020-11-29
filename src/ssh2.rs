use anyhow::{Context, Result};
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

    let (mut listener, _) =
        ssh2_run(|| session.channel_forward_listen(30000, Some("localhost"), None))?;

    let _channel = ssh2_run(|| listener.accept())?;

    handle.join().unwrap()?;

    Ok(())
}

fn ssh2_run<T, F: FnMut() -> std::result::Result<T, ssh2::Error>>(mut fun: F) -> Result<T> {
    loop {
        match fun() {
            Ok(t) => break Ok(t),
            Err(e)
                if std::io::Error::from(ssh2::Error::from_errno(e.code())).kind()
                    == std::io::ErrorKind::WouldBlock => {}
            Err(e) => break Err(e).context("SSH error"),
        }

        thread::sleep(Duration::from_millis(10));
    }
}
