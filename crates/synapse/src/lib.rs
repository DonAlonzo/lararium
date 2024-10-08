mod error;

pub use self::error::{Error, Result};

//use ed25519_dalek::{Signature, Signer, SigningKey};
//use rand::rngs::OsRng;

pub struct Client {}

pub struct Server {}

//pub fn sign(data: &[u8]) -> Signature {
//    let mut prng = OsRng;
//    let signing_key: SigningKey = SigningKey::generate(&mut prng);
//    signing_key.sign(data)
//}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_() {}
}
