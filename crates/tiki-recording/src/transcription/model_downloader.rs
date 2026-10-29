use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use reqwest::StatusCode;
use reqwest::blocking::Client;
use reqwest::header::RANGE;
use sha2::{Digest, Sha256};

use super::TranscriptionModel;
use crate::error::{RecordingError, RecordingResult};
use crate::recording_config::RecordingConfig;

/// How far a model download has come.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
}

/// Keeps Whisper models in a folder: a model is downloaded the first time it is needed,
/// resumes where it stopped if the connection drops, and is kept only when its SHA-256
/// matches the one pinned in `RecordingConfig`.
pub struct ModelDownloader {
    folder: PathBuf,
    http: Client,
}

impl ModelDownloader {
    pub fn new(folder: &Path) -> RecordingResult<Self> {
        Ok(Self {
            folder: folder.to_path_buf(),
            http: Client::builder()
                .connect_timeout(RecordingConfig::DOWNLOAD_CONNECT_TIMEOUT)
                // A model takes minutes to download: only the connection has a deadline.
                .timeout(None)
                .build()?,
        })
    }

    /// The model's file, downloaded first if it is not here yet.
    pub fn ensure(
        &self,
        model: &TranscriptionModel,
        progress: &mut dyn FnMut(DownloadProgress),
    ) -> RecordingResult<PathBuf> {
        let path = self.folder.join(model.file_name);
        if fs::metadata(&path).is_ok_and(|file| file.len() == model.bytes) {
            return Ok(path);
        }
        fs::create_dir_all(&self.folder)?;
        let partial = path.with_extension(RecordingConfig::PARTIAL_DOWNLOAD_EXTENSION);
        let mut hasher = Sha256::new();
        let mut downloaded = Self::hash_file(&partial, &mut hasher)?;
        let mut request = self.http.get(model.url);
        if downloaded > 0 {
            request = request.header(RANGE, format!("bytes={downloaded}-"));
        }
        let mut response = request.send()?;
        let mut file = match response.status() {
            StatusCode::PARTIAL_CONTENT => OpenOptions::new().append(true).open(&partial)?,
            status if status.is_success() => {
                // The server sent the whole file again: start over.
                hasher = Sha256::new();
                downloaded = 0;
                File::create(&partial)?
            }
            status => return Err(RecordingError::DownloadRefused(status.as_u16())),
        };
        let mut buffer = vec![0; RecordingConfig::DOWNLOAD_BUFFER_BYTES];
        loop {
            let read = response.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            file.write_all(&buffer[..read])?;
            hasher.update(&buffer[..read]);
            downloaded += u64::try_from(read).unwrap_or_default();
            progress(DownloadProgress {
                downloaded_bytes: downloaded,
                total_bytes: model.bytes,
            });
        }
        file.flush()?;
        drop(file);
        if downloaded != model.bytes || Self::hex(&hasher.finalize()) != model.sha256 {
            fs::remove_file(&partial)?;
            return Err(RecordingError::ChecksumMismatch);
        }
        fs::rename(&partial, &path)?;
        Ok(path)
    }

    /// Feeds what an interrupted download saved to the hasher; returns its length.
    fn hash_file(path: &Path, hasher: &mut Sha256) -> RecordingResult<u64> {
        let Ok(mut file) = File::open(path) else {
            return Ok(0);
        };
        let mut buffer = vec![0; RecordingConfig::DOWNLOAD_BUFFER_BYTES];
        let mut length = 0;
        loop {
            let read = file.read(&mut buffer)?;
            if read == 0 {
                return Ok(length);
            }
            hasher.update(&buffer[..read]);
            length += u64::try_from(read).unwrap_or_default();
        }
    }

    fn hex(digest: &[u8]) -> String {
        digest.iter().fold(String::new(), |mut hex, byte| {
            // Writing to a String cannot fail.
            let _ = write!(hex, "{byte:02x}");
            hex
        })
    }
}
