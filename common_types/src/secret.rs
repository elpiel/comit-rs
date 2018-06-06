use crypto::digest::Digest;
use crypto::sha2::Sha256;
use hex;
use rand::{OsRng, Rng};
use std::fmt;

const SHA256_DIGEST_LENGTH: usize = 32;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct SecretHash(pub Vec<u8>);

impl fmt::Display for SecretHash {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_str(hex::encode(&self.0).as_str())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Secret {
    secret: [u8; SHA256_DIGEST_LENGTH],
    hash: Option<SecretHash>,
}

impl From<[u8; SHA256_DIGEST_LENGTH]> for Secret {
    fn from(secret: [u8; SHA256_DIGEST_LENGTH]) -> Self {
        Secret { secret, hash: None }
    }
}

impl Secret {
    pub fn generate<T: RandomnessSource>(rng: &mut T) -> Secret {
        let random_bytes = rng.gen_random_bytes(SHA256_DIGEST_LENGTH);
        let mut secret = [0; 32];
        secret.copy_from_slice(&random_bytes[..]);
        Secret::from(secret)
    }

    pub fn hash(&mut self) -> &SecretHash {
        match self.hash {
            None => {
                let mut sha = Sha256::new();
                sha.input(&self.secret);

                let mut result: [u8; SHA256_DIGEST_LENGTH] = [0; SHA256_DIGEST_LENGTH];
                sha.result(&mut result);
                let hash = SecretHash(result.to_vec());

                self.hash = Some(hash.clone());
                self.hash()
            }
            Some(ref hash) => hash,
        }
    }

    pub fn raw_secret(&self) -> &[u8; SHA256_DIGEST_LENGTH] {
        &self.secret
    }
}

pub trait RandomnessSource {
    fn gen_random_bytes(&mut self, nbytes: usize) -> Vec<u8>;
}

impl RandomnessSource for OsRng {
    fn gen_random_bytes(&mut self, nbytes: usize) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0; nbytes];
        self.fill_bytes(&mut buf);
        buf
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::vec::Vec;

    #[test]
    fn gen_random_bytes_not_zeros() {
        let mut rng = OsRng::new().unwrap();

        let empty_buf: Vec<u8> = vec![0; 32];
        let buf = rng.gen_random_bytes(32);
        assert_eq!(buf.len(), 32);
        assert_ne!(buf, empty_buf);
    }

    #[test]
    fn new_secret_hash_as_hex() {
        let bytes = b"hello world, you are beautiful!!";
        let mut secret = Secret::from(*bytes);
        assert_eq!(
            secret.hash().to_string(),
            "68d627971643a6f97f27c58957826fcba853ec2077fd10ec6b93d8e61deb4cec"
        );
    }
}