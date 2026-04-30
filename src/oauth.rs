use anyhow::{Context, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::mpsc::{self, Receiver};

pub const CALLBACK_PORT: u16 = 9876;
pub const CALLBACK_URL: &str = "http://localhost:9876/auth/github/callback";

pub struct PkceChallenge {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

pub fn generate_pkce() -> PkceChallenge {
    let mut buf = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut buf);
    let verifier = URL_SAFE_NO_PAD.encode(buf);

    let mut hasher = Sha256::new();
    hasher.update(verifier.as_bytes());
    let hash = hasher.finalize();
    let challenge = URL_SAFE_NO_PAD.encode(hash.as_slice());

    // Generate a separate random string for state
    let mut state_buf = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut state_buf);
    let state = URL_SAFE_NO_PAD.encode(state_buf);

    PkceChallenge { verifier, challenge, state }
}

/// Starts a local HTTP server on a random port.
/// Returns the port and a channel that will receive the GitHub auth code.
pub fn start_callback_server() -> Result<Receiver<(String, String)>> {
    let listener = TcpListener::bind(format!("127.0.0.1:{}", CALLBACK_PORT))
        .context("Port 9876 is in use. Please free it and try again.")?;
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut buf = [0u8; 8192];
            if let Ok(n) = stream.read(&mut buf) {
                let request = String::from_utf8_lossy(&buf[..n]);
                let code = extract_query_param(&request, "code");
                let state = extract_query_param(&request, "state");

                let html = match &code {
                    Some(_) => include_str!("templates/success.html"),
                    None => include_str!("templates/error.html"),
                };

                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    html.len(),
                    html
                );
                let _ = stream.write_all(response.as_bytes());

                if let (Some(code), Some(state)) = (code, state) {
                    let _ = tx.send((code, state));
                }
            }
        }
    });

    Ok(rx)
}

fn extract_query_param(request: &str, param: &str) -> Option<String> {
    let path = request.lines().next()?.split_whitespace().nth(1)?;
    let query = path.split('?').nth(1)?;
    for pair in query.split('&') {
        let mut kv = pair.splitn(2, '=');
        if kv.next()? == param {
            return Some(kv.next()?.to_string());
        }
    }
    None
}