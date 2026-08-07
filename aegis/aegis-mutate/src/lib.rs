use rand::RngCore;
use rand::seq::SliceRandom;
use aegis_core::{Input, AegisResult};

const INTERESTING_8: &[i8] = &[-128, -1, 0, 1, 16, 32, 64, 100, 127];
const INTERESTING_16: &[i16] = &[-32768, -129, 128, 255, 256, 512, 1000, 1024, 4096, 32767];
const INTERESTING_32: &[i32] = &[-2147483648, -100663046, -32769, 32768, 65535, 65536, 100663039, 2147483647];

#[derive(Debug, Clone)]
pub struct MutatorEngine {
    rng: rand_chacha::ChaCha8Rng,
    mutation_count: u64,
}

impl MutatorEngine {
    pub fn new(seed: u64) -> Self {
        use rand::SeedableRng;
        Self { rng: rand_chacha::ChaCha8Rng::seed_from_u64(seed), mutation_count: 0 }
    }

    pub fn mutate(&mut self, input: &dyn Input) -> AegisResult<Vec<u8>> {
        let mut bytes = input.as_bytes().to_vec();
        if bytes.is_empty() {
            let size = (self.rng.next_u32() % 256) as usize + 1;
            bytes.resize(size, 0);
            for b in bytes.iter_mut() { *b = self.rng.next_u32() as u8; }
            return Ok(bytes);
        }
        let strategy = self.rng.next_u32() % 6;
        match strategy {
            0 => self.bit_flip(&mut bytes),
            1 => self.byte_flip(&mut bytes),
            2 => self.arithmetic(&mut bytes),
            3 => self.interesting(&mut bytes),
            4 => self.insert_delete(&mut bytes),
            _ => self.splice(&mut bytes),
        }
        self.mutation_count += 1;
        Ok(bytes)
    }

    fn bit_flip(&mut self, bytes: &mut [u8]) {
        let bit_idx = (self.rng.next_u32() as usize) % (bytes.len() * 8);
        bytes[bit_idx / 8] ^= 1 << (bit_idx % 8);
    }

    fn byte_flip(&mut self, bytes: &mut [u8]) {
        let num = ((self.rng.next_u32() as usize) % 16) + 1;
        for _ in 0..num.min(bytes.len()) {
            bytes[(self.rng.next_u32() as usize) % bytes.len()] = self.rng.next_u32() as u8;
        }
    }

    fn arithmetic(&mut self, bytes: &mut [u8]) {
        if bytes.len() < 2 { return; }
        let idx = (self.rng.next_u32() as usize) % (bytes.len() - 1);
        let is_add = self.rng.next_u32() % 2 == 0;
        let delta = ((self.rng.next_u32() % 35) + 1) as i64;
        if bytes.len() - idx >= 4 && self.rng.next_u32() % 2 == 0 {
            let val = u32::from_le_bytes([bytes[idx], bytes[idx+1], bytes[idx+2], bytes[idx+3]]) as i64;
            let new_val = if is_add { val.wrapping_add(delta) } else { val.wrapping_sub(delta) };
            bytes[idx..idx+4].copy_from_slice(&(new_val as u32).to_le_bytes());
        } else if bytes.len() - idx >= 2 {
            let val = u16::from_le_bytes([bytes[idx], bytes[idx+1]]) as i64;
            let new_val = if is_add { val.wrapping_add(delta) } else { val.wrapping_sub(delta) };
            bytes[idx..idx+2].copy_from_slice(&(new_val as u16).to_le_bytes());
        } else {
            let val = bytes[idx] as i64;
            bytes[idx] = (if is_add { val.wrapping_add(delta) } else { val.wrapping_sub(delta) }) as u8;
        }
    }

    fn interesting(&mut self, bytes: &mut [u8]) {
        let idx = (self.rng.next_u32() as usize) % bytes.len();
        let size = if bytes.len() - idx >= 4 { 4 } else if bytes.len() - idx >= 2 { 2 } else { 1 };
        match size {
            1 => bytes[idx] = *INTERESTING_8.choose(&mut self.rng).unwrap_or(&0) as u8,
            2 => { let v = *INTERESTING_16.choose(&mut self.rng).unwrap_or(&0); bytes[idx..idx+2].copy_from_slice(&v.to_le_bytes()); }
            _ => { let v = *INTERESTING_32.choose(&mut self.rng).unwrap_or(&0); let end = (idx+4).min(bytes.len()); bytes[idx..end].copy_from_slice(&v.to_le_bytes()[..end-idx]); }
        }
    }

    fn insert_delete(&mut self, bytes: &mut Vec<u8>) {
        let is_insert = self.rng.next_u32() % 2 == 0;
        if is_insert {
            let pos = (self.rng.next_u32() as usize) % (bytes.len() + 1);
            let size = ((self.rng.next_u32() as usize) % 64) + 1;
            let mut new = Vec::with_capacity(bytes.len() + size);
            new.extend_from_slice(&bytes[..pos]);
            for _ in 0..size { new.push(self.rng.next_u32() as u8); }
            new.extend_from_slice(&bytes[pos..]);
            *bytes = new;
        } else {
            let pos = (self.rng.next_u32() as usize) % bytes.len();
            let size = (((self.rng.next_u32() as usize) % 64) + 1).min(bytes.len() - pos);
            bytes.drain(pos..pos+size);
        }
    }

    fn splice(&mut self, bytes: &mut Vec<u8>) {
        if bytes.len() < 4 { return; }
        let pos1 = (self.rng.next_u32() as usize) % (bytes.len() - 1);
        let pos2 = (self.rng.next_u32() as usize) % (bytes.len() - 1);
        let len = (((self.rng.next_u32() as usize) % (bytes.len() / 4)) + 1).min(bytes.len() - pos1.max(pos2));
        if pos1 != pos2 && pos1 + len <= bytes.len() && pos2 + len <= bytes.len() {
            for i in 0..len { bytes.swap(pos1 + i, pos2 + i); }
        }
    }

    pub fn mutation_count(&self) -> u64 { self.mutation_count }
}
