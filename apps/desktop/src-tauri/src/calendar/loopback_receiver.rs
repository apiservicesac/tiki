use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use url::Url;

use super::sign_in_page::SignInPages;

/// Receives the provider's answer in the browser on `http://127.0.0.1:<port>`, the way
/// desktop apps finish signing in. It only listens on this computer and only for one answer.
pub struct LoopbackReceiver {
    listener: TcpListener,
}

impl LoopbackReceiver {
    pub fn bind() -> std::io::Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        Ok(Self { listener })
    }

    pub fn redirect_uri(&self) -> std::io::Result<String> {
        Ok(format!(
            "http://127.0.0.1:{}",
            self.listener.local_addr()?.port()
        ))
    }

    /// Waits for the browser to come back with `code` and the expected `state`, until the
    /// timeout or until `cancelled` is set.
    pub fn wait_for_code(
        &self,
        state: &str,
        timeout: Duration,
        cancelled: &AtomicBool,
        pages: &SignInPages,
    ) -> Result<String, String> {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline {
            if cancelled.load(Ordering::Relaxed) {
                return Err("the sign-in was cancelled".to_owned());
            }
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Some(answer) = Self::read_answer(stream, state, pages) {
                        return answer;
                    }
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    std::thread::sleep(Duration::from_millis(100));
                }
                Err(error) => return Err(error.to_string()),
            }
        }
        Err("the sign-in took too long".to_owned())
    }

    /// The code from one browser request, an error the provider sent, or `None` for an
    /// unrelated request (such as the browser asking for a favicon).
    fn read_answer(
        mut stream: TcpStream,
        state: &str,
        pages: &SignInPages,
    ) -> Option<Result<String, String>> {
        stream.set_nonblocking(false).ok()?;
        let mut request_line = String::new();
        BufReader::new(&stream).read_line(&mut request_line).ok()?;
        let path = request_line.split_whitespace().nth(1)?;
        let url = Url::parse(&format!("http://127.0.0.1{path}")).ok()?;
        let value = |name: &str| {
            url.query_pairs()
                .find(|(key, _)| key == name)
                .map(|(_, value)| value.into_owned())
        };
        let answer = match (value("code"), value("error"), value("state")) {
            (_, Some(error), _) => Err(error),
            (Some(code), None, Some(returned)) if returned == state => Ok(code),
            (Some(_), None, _) => Err("the sign-in answer did not match".to_owned()),
            (None, None, _) => return None,
        };
        let body = pages.for_answer(answer.is_ok());
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        let _ = stream.write_all(response.as_bytes());
        Some(answer)
    }
}
