mod error;

pub use self::error::{Error, Result};

use openssl::asn1::Asn1Time;
use openssl::bn::{BigNum, MsbOption};
use openssl::derive::Deriver;
use openssl::hash::MessageDigest;
use openssl::nid::Nid;
use openssl::pkey::{Id, PKey, Private, Public};
use openssl::sign::{Signer, Verifier};
use openssl::stack::Stack;
use openssl::symm::{Cipher, Crypter, Mode};
use openssl::x509::{
    extension::{BasicConstraints, KeyUsage, SubjectKeyIdentifier},
    store::X509StoreBuilder,
    X509NameBuilder, X509NameRef, X509Req, X509StoreContext, X509,
};

#[derive(Clone)]
pub struct Identity {
    certificate: Certificate,
    private_key: PrivateSignatureKey,
}

#[derive(Clone)]
pub struct Certificate {
    x509: X509,
}

pub struct CertificateSigningRequest {
    x509_req: X509Req,
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

impl Identity {
    pub fn new(name: &str) -> Result<Identity> {
        let private_key = PrivateSignatureKey::new()?;
        let subject_name = {
            let mut name_builder = X509NameBuilder::new()?;
            name_builder.append_entry_by_nid(Nid::COMMONNAME, name)?;
            name_builder.build()
        };
        let certificate = create_certificate(
            &private_key.pkey,
            &private_key.public_key()?.pkey,
            None,
            &subject_name,
        )?;
        Ok(Identity {
            certificate,
            private_key,
        })
    }

    pub fn sign_certificate_signing_request(
        &self,
        csr: &CertificateSigningRequest,
    ) -> Result<Certificate> {
        Ok(create_certificate(
            &self.private_key.pkey,
            &csr.x509_req.public_key()?,
            Some(&self.certificate.x509),
            &csr.x509_req.subject_name(),
        )?)
    }

    pub fn certificate(&self) -> &Certificate {
        &self.certificate
    }

    pub fn sign(
        &self,
        data: &[u8],
    ) -> Result<Signature> {
        self.private_key.sign(data)
    }
}

impl Certificate {
    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        let x509 = X509::from_pem(pem).map_err(|_| Error::InvalidCertificate)?;
        Ok(Self { x509 })
    }

    pub fn to_pem(&self) -> Result<Vec<u8>> {
        self.x509.to_pem().map_err(|_| Error::InvalidCertificate)
    }

    pub fn public_key(&self) -> Result<PublicSignatureKey> {
        let public_key = self.x509.public_key()?;
        Ok(PublicSignatureKey { pkey: public_key })
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

    pub fn verify(
        &self,
        data: &[u8],
        signature: &Signature,
    ) -> Result<bool> {
        let public_key = self.x509.public_key()?;
        let mut verifier = Verifier::new_without_digest(&public_key)?;
        let verified = verifier.verify_oneshot(signature.raw.as_slice(), data)?;
        Ok(verified)
    }
}

impl CertificateSigningRequest {
    pub fn from_pem(pem: &[u8]) -> Result<Self> {
        let x509_req = X509Req::from_pem(pem).map_err(|_| Error::InvalidCertificate)?;
        Ok(Self { x509_req })
    }

    pub fn to_pem(&self) -> Result<Vec<u8>> {
        self.x509_req
            .to_pem()
            .map_err(|_| Error::InvalidCertificate)
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

    pub fn to_pem(&self) -> Result<Vec<u8>> {
        self.pkey
            .private_key_to_pem_pkcs8()
            .map_err(|_| Error::InvalidPrivateKey)
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        self.pkey
            .raw_private_key()
            .map_err(|_| Error::InvalidPrivateKey)
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

    pub fn to_pem(&self) -> Result<Vec<u8>> {
        self.pkey
            .private_key_to_pem_pkcs8()
            .map_err(|_| Error::InvalidPrivateKey)
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        self.pkey
            .raw_private_key()
            .map_err(|_| Error::InvalidPrivateKey)
    }

    pub fn into_identity(
        self,
        certificate: Certificate,
    ) -> Result<Identity> {
        let certificate_public_key = certificate.public_key()?.pkey.public_key_to_der()?;
        let public_key = self.public_key()?.pkey.public_key_to_der()?;
        if certificate_public_key != public_key {
            return Err(Error::InvalidCertificate);
        }
        Ok(Identity {
            certificate,
            private_key: self,
        })
    }

    pub fn public_key(&self) -> Result<PublicSignatureKey> {
        let public_key = self.pkey.raw_public_key()?;
        let public_key = PKey::public_key_from_raw_bytes(&public_key, Id::ED25519)?;
        Ok(PublicSignatureKey { pkey: public_key })
    }

    pub fn generate_certificate_signing_request(&self) -> Result<CertificateSigningRequest> {
        let subject_name = {
            let mut name = X509NameBuilder::new()?;
            name.append_entry_by_nid(Nid::COMMONNAME, "random-name")?;
            name.build()
        };
        let mut x509_req = X509Req::builder()?;
        x509_req.set_subject_name(&subject_name)?;
        x509_req.set_pubkey(&self.public_key()?.pkey)?;
        x509_req.sign(&self.pkey, MessageDigest::null())?;
        let x509_req = x509_req.build();
        Ok(CertificateSigningRequest { x509_req })
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

    pub fn to_pem(&self) -> Result<Vec<u8>> {
        self.pkey
            .public_key_to_pem()
            .map_err(|_| Error::InvalidPublicKey)
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        self.pkey
            .raw_public_key()
            .map_err(|_| Error::InvalidPublicKey)
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

    pub fn to_pem(&self) -> Result<Vec<u8>> {
        self.pkey
            .public_key_to_pem()
            .map_err(|_| Error::InvalidPublicKey)
    }

    pub fn to_raw(&self) -> Result<Vec<u8>> {
        self.pkey
            .raw_public_key()
            .map_err(|_| Error::InvalidPublicKey)
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

fn create_certificate(
    signing_key: &PKey<Private>,
    public_key: &PKey<Public>,
    issuer: Option<&X509>,
    subject_name: &X509NameRef,
) -> Result<Certificate> {
    let serial_number = {
        let mut serial = BigNum::new()?;
        serial.rand(159, MsbOption::MAYBE_ZERO, false)?;
        serial.to_asn1_integer()?
    };
    let not_before = Asn1Time::days_from_now(0)?;
    let not_after = Asn1Time::days_from_now(7300)?;
    let basic_constraints = BasicConstraints::new().critical().ca().build()?;
    let key_usage = KeyUsage::new()
        .critical()
        .key_cert_sign()
        .crl_sign()
        .build()?;

    let mut x509 = X509::builder()?;
    x509.set_version(2)?;
    x509.set_subject_name(subject_name)?;
    x509.set_issuer_name({
        if let Some(issuer) = issuer {
            issuer.subject_name()
        } else {
            subject_name
        }
    })?;
    x509.set_pubkey(&public_key)?;
    x509.set_not_before(&not_before)?;
    x509.set_not_after(&not_after)?;
    x509.set_serial_number(&serial_number)?;
    x509.append_extension(key_usage)?;
    x509.append_extension(basic_constraints)?;
    x509.append_extension(
        SubjectKeyIdentifier::new()
            .build(&x509.x509v3_context(issuer.map(|issuer| &**issuer), None))?,
    )?;
    x509.sign(&signing_key, MessageDigest::null())?;
    let x509 = x509.build();
    Ok(Certificate { x509 })
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
    fn verify_certificate() {
        let root = Identity::new("owner ca").unwrap();

        let controller_private_key = PrivateSignatureKey::new().unwrap();
        let controller_csr = controller_private_key
            .generate_certificate_signing_request()
            .unwrap();
        let controller_certificate = root
            .sign_certificate_signing_request(&controller_csr)
            .unwrap();
        let controller = controller_private_key
            .into_identity(controller_certificate)
            .unwrap();

        let device_private_key = PrivateSignatureKey::new().unwrap();
        let device_csr = device_private_key
            .generate_certificate_signing_request()
            .unwrap();
        let device_certificate = controller
            .sign_certificate_signing_request(&device_csr)
            .unwrap();
        let device = device_private_key
            .into_identity(device_certificate)
            .unwrap();

        let verified = Certificate::verify_chain(
            root.certificate(),
            &[controller.certificate(), device.certificate()],
        )
        .unwrap();
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