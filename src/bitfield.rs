pub struct Bitfield {
  bits: Vec<u8>,
  num_bits: usize,
}

fn blocks_for_nbits(n: usize) -> usize {
  (n + 7) / 8
}

impl Bitfield {
  fn bit_block_and_offset(&self, index: usize) -> (usize, u8) {
    (index / 8, index as u8 % 8)
  }

  fn num_blocks(&self) -> usize {
    self.bits.len()
  }

  pub fn new(n: usize) -> Self {
    let num_blocks = blocks_for_nbits(n);
    Self {
      bits: vec![0; num_blocks],
      num_bits: n,
    }
  }

  pub fn from_bytes(bytes: &[u8]) -> Self {
    Self {
      bits: Vec::from(bytes),
      num_bits: bytes.len() * 8,
    }
  }

  pub fn as_bytes(&self) -> &[u8] {
    self.bits.as_slice()
  }

  pub fn set(&mut self, index: usize) {
    debug_assert!(index < self.num_bits, "bound check failed");
    let (block_index, bit_offset) = self.bit_block_and_offset(index);
    self.bits[block_index] |= 1 << bit_offset;
  }

  pub fn get(&self, index: usize) -> bool {
    debug_assert!(index < self.num_bits, "bound check failed");
    let (block_index, bit_offset) = self.bit_block_and_offset(index);
    (self.bits[block_index] & (1 << bit_offset)) != 0
  }

  pub fn all(&self) -> bool {
    for x in &self.bits {
      if *x != !0u8 {
        return false;
      }
    }
    true
  }

  pub fn bytes(&self) -> u32 {
    self.bits.len() as u32
  }
}

impl Default for Bitfield {
  fn default() -> Self {
    Self::new(0)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn creation_bitfield() {
    let bf = Bitfield::new(8);
    assert_eq!(bf.num_bits, 8);
    assert_eq!(bf.bits.len(), 1);

    let bf_from_x = Bitfield::from_bytes(&[0b10010010]);
    assert_eq!(bf_from_x.num_bits, 8);
    assert_eq!(bf_from_x.bits.len(), 1);

    let bf_from_y = Bitfield::from_bytes(&[0b10010010, 0b10010010]);
    assert_eq!(bf_from_y.num_bits, 16);
    assert_eq!(bf_from_y.bits.len(), 2);
  }

  #[test]
  fn set_get_bit() {
    let mut bf = Bitfield::from_bytes(&[0b00000000]);
    assert!(!bf.get(0));
    bf.set(0);
    assert!(bf.get(0));
    bf.set(7);
    assert!(bf.get(7));
  }

  #[test]
  fn all_set() {
    let mut bf1 = Bitfield::from_bytes(&[0b00000000]);
    let mut bf2 = Bitfield::from_bytes(&[!0u8]);

    assert!(!bf1.all());
    assert!(bf2.all());
  }
}
