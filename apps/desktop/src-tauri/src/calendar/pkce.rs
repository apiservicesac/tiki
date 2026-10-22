use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};

/// The proof that the app finishing the sign-in is the one that started it (RFC 7636),
/// and a random state that ties the browser's answer to this attempt.
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
    pub state: String,
}

impl Pkce {
    pub fn new() -> Result<Self, getrandom::Error> {
        let verifier = Self::random_text(48)?;
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Ok(Self {
            verifier,
            challenge,
            state: Self::random_text(24)?,
        })
    }

    fn random_text(bytes: usize) -> Result<String, getrandom::Error> {
        let mut buffer = vec![0_u8; bytes];
        getrandom::fill(&mut buffer)?;
        Ok(URL_SAFE_NO_PAD.encode(buffer))
    }
}
