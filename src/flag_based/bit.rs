use super::FlagVec;
use std::mem::size_of;

pub type Block = u32;
pub const BITS_PER_BYTE: usize = 8;
pub const BITS_PER_BLOCK: usize = size_of::<Block>() * BITS_PER_BYTE;

#[derive(Clone, Debug)]
pub struct BitVec {
    pub flags: Vec<Block>,
    pub num_bits: usize,
}

impl BitVec {
    pub fn new() -> Self {
        return Self {
            flags: Vec::new(),
            num_bits: 0,
        }
    }

    pub fn with_bits(num_bits: usize, value: bool) -> Self {
        if num_bits == 0 {
            return Self{
                flags: Vec::new(),
                num_bits
            }
        }
        
        let value: Block = if value { Block::MAX } else { 0 };
        let num_blocks: usize = ((num_bits-1)/BITS_PER_BLOCK)+1;
        return Self {
            flags: vec![value; num_blocks],
            num_bits
        }
    }

    pub fn num_bits(&self) -> usize {
        return self.num_bits
    }

    pub fn num_blocks(&self) -> usize {
        return self.flags.len()
    }

    pub fn get_bit(&self, bit: usize) -> bool {
        let bit_idx: usize = bit % BITS_PER_BLOCK;
        let flags_idx: usize = bit / BITS_PER_BLOCK;
        assert!(bit < self.num_bits);
        assert!(flags_idx < self.flags.len());
        let bit: bool = (self.flags[flags_idx] & (1 << bit_idx)) != 0;
        return bit
    }

    pub fn set_bit(&mut self, bit: usize, value: bool) {
        let bit_idx: usize = bit % BITS_PER_BLOCK;
        let flag_idx: usize = bit / BITS_PER_BLOCK;
        assert!(bit < self.num_bits);
        assert!(flag_idx < self.flags.len());
        let value: Block = if value { 1 } else { 0 };
        self.flags[flag_idx] &= !(1 << bit_idx); // clear
        self.flags[flag_idx] |= value << bit_idx; // set
    }

    pub fn set_bit_and_all_bits_after_it_to_true(&mut self, bit: usize) {
        assert!(bit < self.num_bits);
        let bit_idx: usize = bit % BITS_PER_BLOCK;
        let flag_idx: usize = bit / BITS_PER_BLOCK;
        assert!(flag_idx < self.flags.len());

        let set_bit_and_bits_after_it_in_to_true: Block = Block::MAX << bit_idx;
        self.flags[flag_idx] |= set_bit_and_bits_after_it_in_to_true;

        for i in (flag_idx+1)..self.flags.len() {
            self.flags[i] = Block::MAX;
        }
    }

    pub fn add_bits(&mut self, num_bits_to_add: usize, value_of_bits: bool) {
        assert!(self.flags.len()*BITS_PER_BLOCK >= self.num_bits);

        let new_num_bits: usize = self.num_bits + num_bits_to_add;
        if new_num_bits == 0 {
            return
        }

        if num_bits_to_add == 0 {
            return
        }

        if self.num_bits == 0 {
            *self = Self::with_bits(num_bits_to_add, value_of_bits);
            return
        }

        let num_alloc_bits_in_last_flag: usize = self.num_bits - ((self.flags.len()-1) * BITS_PER_BLOCK);
        let num_free_bits_in_last_flag: usize = BITS_PER_BLOCK - num_alloc_bits_in_last_flag;
        let num_bits_to_add_from_last_flag: usize =
            if num_bits_to_add <= num_free_bits_in_last_flag { 
                num_bits_to_add
            }
            else {
                num_free_bits_in_last_flag
            };

        let len: usize = self.flags.len()-1;
        let value_of_bits_as_num: Block = if value_of_bits { 1 } else { 0 };
        for bit in num_alloc_bits_in_last_flag..(num_alloc_bits_in_last_flag + num_bits_to_add_from_last_flag) {
            self.flags[len] &= !(1 << bit); // clear
            self.flags[len] |= value_of_bits_as_num << bit; // set
        }

        if num_bits_to_add > num_free_bits_in_last_flag {
            let num_bits_to_add_from_new_flags: usize = num_bits_to_add - num_free_bits_in_last_flag;
            let num_new_flags: usize = ((num_bits_to_add_from_new_flags-1)/BITS_PER_BLOCK)+1;
            let new_num_blocks: usize = self.flags.len() + num_new_flags;
            let value_of_new_flags: Block = if value_of_bits { Block::MAX } else { 0 };
            self.flags.resize(new_num_blocks, value_of_new_flags);
        }

        self.num_bits = new_num_bits;
    }

    pub fn get_block(&self, idx_of_block: usize) -> Block {
        if self.flags.is_empty() {
            return 0
        }

        if idx_of_block == self.flags.len()-1 {
            let mask_out_unallocated_bits: Block;
            let there_are_unallocated_bits: bool = self.num_bits % BITS_PER_BLOCK != 0;
            if there_are_unallocated_bits {
                let idx_of_first_unalloc_bit: Block = ( self.num_bits - BITS_PER_BLOCK*(self.flags.len()-1) ).try_into().unwrap();
                assert!(idx_of_first_unalloc_bit >= 1);
                assert!(idx_of_first_unalloc_bit < BITS_PER_BLOCK as Block);
                mask_out_unallocated_bits = (((1 as Block) << idx_of_first_unalloc_bit)-1) as Block;
            }
            else {
                mask_out_unallocated_bits = Block::MAX; // mask out nothing
            };

            let flags: Block = self.flags[idx_of_block] & mask_out_unallocated_bits;
            return flags
        }

        return self.flags[idx_of_block]
    }

    pub fn _get_num_blocks(&self) -> usize {
        return self.flags.len()
    }

    pub fn _get_block_that_contains_bit(&self, bit: usize) -> usize {
        return bit/BITS_PER_BLOCK
    }

    pub fn find_a_true_bit(&self) -> Option<usize> {
        for block in 0..self.flags.len() {
            if self.flags[block] != 0 {
                let local_bit: usize = self.flags[block].trailing_zeros() as usize;
                let bit: usize = block*BITS_PER_BLOCK + local_bit;
                return Some(bit)
            }
        }
        return None
    }

    pub fn true_bits<'a>(&'a self) -> TrueBitsIterator<'a> {
        return TrueBitsIterator::new(self)
    }
}

pub struct TrueBitsIterator<'a> {
    bits: &'a BitVec,
    bit: usize,
}

impl <'a> TrueBitsIterator<'a> {
    fn new(bits: &'a BitVec) -> Self {
        return Self {
            bits,
            bit: 0,
        }
    }
}

impl <'a> Iterator for TrueBitsIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if self.bit >= self.bits.num_bits() {
                return None
            }

            if self.bits.get_bit(self.bit) == false {
                self.bit += 1;
            }
            else {
                break;
            }
        }

        let true_bit: usize = self.bit;
        self.bit += 1;
        return Some(true_bit)
    }
}

impl FlagVec for BitVec {
    type TrueFlagsIter<'a> = TrueBitsIterator<'a>;

    fn new() -> Self {
        return Self::new()
    }

    fn with_flags(num_flags: usize, value: bool) -> Self {
        return Self::with_bits(num_flags, value)
    }
    
    fn num_flags(&self) -> usize {
        return self.num_bits()
    }

    fn get_flag(&self, flag: usize) -> bool {
        return self.get_bit(flag)
    }

    fn set_flag(&mut self, flag: usize, value: bool) {
        self.set_bit(flag, value);
    }

    fn add_flags(&mut self, num_flags: usize, value: bool) {
        self.add_bits(num_flags, value);
    }

    fn find_a_true_flag(&self) -> Option<usize> {
        return self.find_a_true_bit()
    }

    fn true_flags<'a>(&'a self) -> Self::TrueFlagsIter<'a> {
        return self.true_bits()
    }
}