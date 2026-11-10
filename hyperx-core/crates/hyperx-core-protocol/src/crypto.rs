use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes128;

/// AES/CFB8 stream state for Minecraft network encryption/decryption
#[derive(Clone)]
pub struct AesCfb8 {
    cipher: Aes128,
    register: [u8; 16],
}

impl AesCfb8 {
    pub fn new(key: &[u8; 16], iv: &[u8; 16]) -> Self {
        Self {
            cipher: Aes128::new_from_slice(key).expect("Invalid key length"),
            register: *iv,
        }
    }

    fn keystream(&self) -> u8 {
        let mut block = self.register.into();
        self.cipher.encrypt_block(&mut block);
        let block_bytes: [u8; 16] = block.into();
        block_bytes[0]
    }

    fn shift(&mut self, cipher_byte: u8) {
        self.register.copy_within(1.., 0);
        self.register[15] = cipher_byte;
    }

    pub fn encrypt(&mut self, data: &mut [u8]) {
        for b in data {
            *b ^= self.keystream();
            self.shift(*b);
        }
    }

    pub fn decrypt(&mut self, data: &mut [u8]) {
        for b in data {
            let c = *b;
            *b ^= self.keystream();
            self.shift(c);
        }
    }
}
