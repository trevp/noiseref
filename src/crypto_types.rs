use utils::*;
use constants::*;

pub trait RandomType {
    fn fill_bytes(&mut self, out: &mut [u8]);
}

pub trait DhType {
    fn name(&self, out: &mut [u8]) -> usize;
    fn pub_len(&self) -> usize;

    fn set(&mut self, privkey: &[u8], pubkey: &[u8]);
    fn generate(&mut self, rng: &mut RandomType); 
    fn pubkey(&self) -> &[u8];
    fn dh(&self, pubkey: &[u8], out: &mut [u8]);
}

pub trait CipherType {
    fn name(&self, out: &mut [u8]) -> usize;

    fn set(&mut self, key: &[u8]);
    fn encrypt(&self, nonce: u64, authtext: &[u8], plaintext: &[u8], out: &mut[u8]);
    fn decrypt(&self, nonce: u64, authtext: &[u8], ciphertext: &[u8], out: &mut[u8]) -> bool;
}

pub trait HashType {
    fn name(&self, out: &mut [u8]) -> usize;
    fn block_len(&self) -> usize;
    fn hash_len(&self) -> usize;

    /* These functions operate on internal state:
     * call reset(), then input() repeatedly, then get result() */
    fn reset(&mut self);
    fn input(&mut self, data: &[u8]);
    fn result(&mut self, out: &mut [u8]);

    /* The hmac and hkdf functions modify internal state
     * but ignore previous state, they're one-shot, static-like functions */
    fn hmac(&mut self, key: &[u8], data: &[u8], out: &mut [u8]) {
        assert!(key.len() <= self.block_len());
        let block_len = self.block_len();
        let hash_len = self.hash_len();
        let mut i_pad = [0x36u8; MAXBLOCKLEN];
        let mut o_pad = [0x5cu8; MAXBLOCKLEN];
        for count in 0..key.len() {
            i_pad[count] ^= key[count];
            o_pad[count] ^= key[count];
        }
        self.reset();
        self.input(&i_pad[..block_len]);
        self.input(data);
        let mut inner_output = [0u8; MAXHASHLEN];
        self.result(&mut inner_output);
        self.reset();
        self.input(&o_pad[..block_len]);
        self.input(&inner_output[..hash_len]);
        self.result(out);
    }

    fn hkdf(&mut self, chaining_key: &[u8], input_key_material: &[u8], out1: &mut [u8], out2: & mut[u8]) {
        let hash_len = self.hash_len();
        let mut temp_key = [0u8; MAXHASHLEN];
        let mut in2 = [0u8; MAXHASHLEN+1];
        self.hmac(chaining_key, input_key_material, &mut temp_key);
        self.hmac(&temp_key, &[1u8], out1);
        copy_memory(&out1[0..hash_len], &mut in2);
        in2[hash_len] = 2;
        self.hmac(&temp_key, &in2[..hash_len+1], out2);
    }
}
