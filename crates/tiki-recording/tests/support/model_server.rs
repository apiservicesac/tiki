use std::io::{BufRead, BufReader, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;

/// How the fake server answers.
#[derive(Clone, Copy)]
pub struct ServerBehavior {
    /// The first answer stops after this many bytes, as a dropped connection would.
    pub cut_first_after: Option<usize>,
    /// Whether it answers a `Range` request with the rest of the file (206) or ignores it.
    pub honors_ranges: bool,
}

/// A model server on localhost that serves one file over plain HTTP/1.1.
pub struct FakeModelServer {
    pub url: &'static str,
    /// The first byte each request asked for (`None` without a `Range` header).
    requested_from: Arc<Mutex<Vec<Option<usize>>>>,
}

impl FakeModelServer {
    pub fn start(body: Vec<u8>, behavior: ServerBehavior) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("local port");
        let address = listener.local_addr().expect("local address");
        let requested_from = Arc::new(Mutex::new(Vec::new()));
        let recorded = Arc::clone(&requested_from);
        thread::spawn(move || {
            for (index, stream) in listener.incoming().enumerate() {
                let Ok(stream) = stream else { continue };
                let cut = if index == 0 {
                    behavior.cut_first_after
                } else {
                    None
                };
                Self::answer(stream, &body, behavior.honors_ranges, cut, &recorded);
            }
        });
        let url = format!("http://{address}/ggml-test.bin");
        Self {
            url: Box::leak(url.into_boxed_str()),
            requested_from,
        }
    }

    pub fn requests(&self) -> Vec<Option<usize>> {
        self.requested_from
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn answer(
        mut stream: TcpStream,
        body: &[u8],
        honors_ranges: bool,
        cut: Option<usize>,
        recorded: &Mutex<Vec<Option<usize>>>,
    ) {
        let mut range_start = None;
        let mut reader = BufReader::new(stream.try_clone().expect("stream"));
        let mut line = String::new();
        while reader.read_line(&mut line).is_ok_and(|read| read > 0) && line != "\r\n" {
            if let Some(range) = line.to_ascii_lowercase().strip_prefix("range: bytes=") {
                range_start = range.trim().trim_end_matches('-').parse().ok();
            }
            line.clear();
        }
        recorded
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .push(range_start);
        let start = if honors_ranges {
            range_start.unwrap_or(0)
        } else {
            0
        };
        let rest = &body[start..];
        let head = if start > 0 {
            format!(
                "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {start}-{}/{}\r\nConnection: close\r\n\r\n",
                rest.len(),
                body.len() - 1,
                body.len()
            )
        } else {
            format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                rest.len()
            )
        };
        let sent = cut.map_or(rest, |cut| &rest[..cut]);
        let _ = stream.write_all(head.as_bytes());
        let _ = stream.write_all(sent);
        let _ = stream.flush();
    }
}
