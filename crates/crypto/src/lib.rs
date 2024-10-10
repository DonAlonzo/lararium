mod error;

pub use self::error::{Error, Result};

use openssl::derive::Deriver;
use openssl::hash::MessageDigest;
use openssl::pkey::{Id, PKey, Private, Public};
use openssl::sign::{Signer, Verifier};
use openssl::stack::Stack;
use openssl::symm::{Cipher, Crypter, Mode};
use openssl::x509::{store::X509StoreBuilder, X509StoreContext, X509};

#[derive(Clone)]
pub struct Certificate {
    x509: X509,
}

#[derive(Clone)]
pub struct PrivateAgreementKey {
    pkey: PKey<Private>,
}

#[derive(Clone)]
pub struct PrivateSignatureKey {
    pkey: PKey<Private>,
}

#[derive(Clone)]
pub struct PublicAgreementKey {
    pkey: PKey<Public>,
}

#[derive(Clone)]
pub struct PublicSignatureKey {
    pkey: PKey<Public>,
}

#[derive(Clone)]
pub struct Signature {
    raw: Vec<u8>,
}

#[derive(Clone)]
pub struct Secret {
    raw: [u8; 32],
}

#[derive(Clone)]
pub struct Encrypted {
    ciphertext: Vec<u8>,
    iv: [u8; 12],
    tag: [u8; 16],
}

impl Certificate {
    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        let x509 = X509::from_pem(pem).map_err(|_| Error::InvalidCertificate)?;
        Ok(Self { x509 })
    }

    pub fn verify_chain(
        ca: &Certificate,
        chain: &[&Certificate],
    ) -> Result<bool> {
        let mut store = X509StoreBuilder::new()?;
        store.add_cert(ca.x509.clone())?;
        let store = store.build();
        let mut stack = Stack::new()?;
        for certificate in chain {
            stack.push(certificate.x509.clone())?;
        }
        let mut store_ctx = X509StoreContext::new()?;
        let verified = store_ctx.init(&store, &chain[0].x509, &stack, |certificate| {
            certificate.verify_cert()
        })?;
        Ok(verified)
    }

    pub fn verify_signature(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool> {
        let public_key = self.x509.public_key()?;
        let digest = MessageDigest::sha256();
        let mut verifier = Verifier::new(digest, &public_key)?;
        verifier.update(data)?;
        let verified = verifier.verify(&signature.raw)?;
        Ok(verified)
    }
}

impl PrivateAgreementKey {
    pub fn new() -> Result<Self> {
        let pkey = PKey::generate_x25519()?;
        Ok(Self { pkey })
    }

    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        match PKey::private_key_from_pem(pem) {
            Ok(pkey) if pkey.id() == Id::X25519 => Ok(Self { pkey }),
            _ => Err(Error::InvalidPrivateKey),
        }
    }

    pub fn public_key(&self) -> Result<PublicAgreementKey> {
        let public_key = self.pkey.raw_public_key()?;
        let public_key = PKey::public_key_from_raw_bytes(&public_key, Id::X25519)?;
        Ok(PublicAgreementKey { pkey: public_key })
    }

    pub fn agree(
        &self,
        public_key: &PublicAgreementKey,
    ) -> Result<Secret> {
        agree(self, public_key)
    }
}

impl PrivateSignatureKey {
    pub fn new() -> Result<Self> {
        let pkey = PKey::generate_ed25519()?;
        Ok(Self { pkey })
    }

    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        match PKey::private_key_from_pem(pem) {
            Ok(pkey) if pkey.id() == Id::ED25519 => Ok(Self { pkey }),
            _ => Err(Error::InvalidPrivateKey),
        }
    }

    pub fn public_key(&self) -> Result<PublicSignatureKey> {
        let public_key = self.pkey.raw_public_key()?;
        let public_key = PKey::public_key_from_raw_bytes(&public_key, Id::ED25519)?;
        Ok(PublicSignatureKey { pkey: public_key })
    }

    pub fn sign(
        &self,
        data: &[u8],
    ) -> Result<Signature> {
        let mut signer = Signer::new_without_digest(&self.pkey)?;
        let signature = signer.sign_oneshot_to_vec(data)?;
        Ok(Signature { raw: signature })
    }
}

impl PublicAgreementKey {
    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        match PKey::public_key_from_pem(pem) {
            Ok(pkey) if pkey.id() == Id::X25519 => Ok(Self { pkey }),
            _ => Err(Error::InvalidPublicKey),
        }
    }

    pub fn agree(
        &self,
        private_key: &PrivateAgreementKey,
    ) -> Result<Secret> {
        agree(private_key, self)
    }
}

impl PublicSignatureKey {
    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        match PKey::public_key_from_pem(pem) {
            Ok(pkey) if pkey.id() == Id::ED25519 => Ok(Self { pkey }),
            _ => Err(Error::InvalidPublicKey),
        }
    }

    pub fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool> {
        let mut verifier = Verifier::new_without_digest(&self.pkey)?;
        let verified = verifier.verify_oneshot(signature.raw.as_slice(), data)?;
        Ok(verified)
    }
}

impl Secret {
    pub fn new() -> Result<Self> {
        let raw = random_bytes::<32>()?;
        Ok(Self { raw })
    }

    pub fn encrypt(
        &self,
        data: &[u8],
        aad: Option<&[u8]>,
    ) -> Result<Encrypted> {
        let cipher = Cipher::aes_256_gcm();
        let iv = random_bytes::<12>()?;
        let mut crypter = Crypter::new(cipher, Mode::Encrypt, &self.raw, Some(&iv))?;
        if let Some(aad) = aad {
            crypter.aad_update(aad)?;
        }
        let mut ciphertext = vec![0; data.len() + cipher.block_size()];
        let mut count = crypter.update(data, &mut ciphertext)?;
        count += crypter.finalize(&mut ciphertext[count..])?;
        let mut tag = [0; 16];
        crypter.get_tag(&mut tag)?;
        ciphertext.truncate(count);
        Ok(Encrypted {
            ciphertext,
            iv,
            tag,
        })
    }
}

impl Encrypted {
    pub fn decrypt(
        &self,
        secret: &Secret,
        aad: Option<&[u8]>,
    ) -> Result<Vec<u8>> {
        let cipher = Cipher::aes_256_gcm();
        let mut crypter = Crypter::new(cipher, Mode::Decrypt, &secret.raw, Some(&self.iv))?;
        if let Some(aad) = aad {
            crypter.aad_update(aad)?;
        }
        crypter.set_tag(&self.tag)?;
        let mut plaintext = vec![0; self.ciphertext.len() + cipher.block_size()];
        let mut count = crypter.update(&self.ciphertext, &mut plaintext)?;
        count += crypter.finalize(&mut plaintext[count..])?;
        plaintext.truncate(count);
        Ok(plaintext)
    }
}

fn agree(
    private_key: &PrivateAgreementKey,
    public_key: &PublicAgreementKey,
) -> Result<Secret> {
    let mut deriver = Deriver::new(&private_key.pkey)?;
    deriver.set_peer(&public_key.pkey)?;
    let mut raw = [0; 32];
    deriver.derive(&mut raw)?;
    Ok(Secret { raw })
}

pub fn random_bytes<const N: usize>() -> Result<[u8; N]> {
    let mut bytes = [0; N];
    openssl::rand::rand_bytes(&mut bytes)?;
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn certificate_from_pem() {
        let pem = include_bytes!("../../../.dev/controller.pem");
        Certificate::from_pem(pem).unwrap();
    }

    #[test]
    fn private_key_from_pem() {
        let pem = include_bytes!("../../../.dev/controller.key");
        PrivateSignatureKey::from_pem(pem).unwrap();
    }

    #[test]
    fn verify_certificate() {
        let ca = include_bytes!("../../../.dev/ca.pem");
        let controller = include_bytes!("../../../.dev/controller.pem");
        let ca = Certificate::from_pem(ca).unwrap();
        let controller = Certificate::from_pem(controller).unwrap();
        let verified = Certificate::verify_chain(&ca, &[&controller]).unwrap();
        assert!(verified);
    }

    #[test]
    fn agree() {
        let private_key = PrivateAgreementKey::new().unwrap();
        let public_key = private_key.public_key().unwrap();
        let secret_1 = private_key.agree(&public_key).unwrap();
        let secret_2 = public_key.agree(&private_key).unwrap();
        assert_eq!(secret_1.raw, secret_2.raw);
        assert_ne!(secret_1.raw, [0; 32]);
        assert_eq!(secret_1.raw.len(), 32);
    }

    #[test]
    fn encrypt_decrypt() {
        let private_key = PrivateAgreementKey::new().unwrap();
        let public_key = private_key.public_key().unwrap();
        let data = b"hello, world!";
        let secret = private_key.agree(&public_key).unwrap();
        let decrypted = secret
            .encrypt(data, None)
            .unwrap()
            .decrypt(&secret, None)
            .unwrap();
        assert_eq!(data, decrypted.as_slice());
    }

    #[test]
    fn bad_aad() {
        let private_key = PrivateAgreementKey::new().unwrap();
        let public_key = private_key.public_key().unwrap();
        let data = b"hello, world!";
        let secret = private_key.agree(&public_key).unwrap();
        let blob = secret.encrypt(data, None).unwrap();
        let result = blob.decrypt(&secret, Some(b"bad aad"));
        assert!(result.is_err());
    }

    #[test]
    fn sign_verify() {
        let private_key = PrivateSignatureKey::new().unwrap();
        let public_key = private_key.public_key().unwrap();
        let data = b"hello, world!";
        let signature = private_key.sign(data).unwrap();
        let verified = public_key.verify(data, &signature).unwrap();
        assert!(verified);
    }
}
