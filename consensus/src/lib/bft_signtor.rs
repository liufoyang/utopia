extern crate crypto;
use crypto::ed25519;

extern crate rustc_hex;
use rustc_hex::{FromHex, ToHex};
extern crate flexi_logger;
use flexi_logger::{Logger, opt_format};
use log::*;

pub struct Bft_Signtor {
    private_key:[u8; 64],
    public_key:[u8; 32],
    seed:String
}

impl Bft_Signtor {
    pub fn new() -> Bft_Signtor{
        let seed: [u8; 32] = [0x26, 0x27, 0xf6, 0x85, 0x97, 0x15, 0xad, 0x1d, 0xd2, 0x94, 0xdd, 0xc4, 0x76, 0x19, 0x39, 0x31,
            0xf1, 0xad, 0xb5, 0x58, 0xf0, 0x93, 0x97, 0x32, 0x19, 0x2b, 0xd1, 0xc0, 0xfd, 0x16, 0x8e, 0x4e];

        //let seed_buf = seed.as_bytes();

        let (priv_buf, public_buf) = ed25519::keypair(&seed);

        let signtor = Bft_Signtor {
            private_key:priv_buf,
            public_key:public_buf,
            seed:seed.to_hex()
        };

        return signtor;
    }

    pub fn get_public_key(&self) -> String {
        let publicKey = self.public_key.to_hex();
        return publicKey;
    }

    pub fn sign_string(&self, payload:&str) -> String {

        let payload_buf = payload.as_bytes();

        let sign_buf = ed25519::signature(payload_buf.as_ref(), self.private_key.as_ref());

        //println!("check by self {}", ed25519::verify(&payload_buf, self.public_key.as_ref(), sign_buf.as_ref()));

        let sign_str = sign_buf.to_hex();
        return sign_str;
    }

    pub fn check_sign(payload:&str, public_key:&str, sign:&str) -> bool {

        let source_sign_vec: Vec<u8> = sign.to_string().from_hex().unwrap();
        let public_key_vec: Vec<u8> = public_key.to_string().from_hex().unwrap();
        let payload_buf = payload.as_bytes();

        return ed25519::verify(&payload_buf, public_key_vec.as_slice(), source_sign_vec.as_slice());
    }
}

#[cfg(test)]
mod tests {
    use super::Bft_Signtor;

    #[test]
    fn test_sign() {
        let signtor = Bft_Signtor::new();

        let msg = "test msg for bft node";

        let sign = signtor.sign_string(msg);

        //info!("check result {}", Bft_Signtor::check_sign(msg, signtor.get_public_key().as_str(), sign.as_str()));

    }
}