use super::FlagVec;
use super::bit::{BITS_PER_BLOCK, Block, BitVec};

/*
    This struct is a bunch of BitVecs stacked on top of each other.    
    The value of a bit in the lowest level BitVec is set by the set_bit() 
    function. For every other level, the value of a bit is derived from 
    a block of bits in the level below it. If any of the bits in that block 
    are ones, then the bit's value is one. Otherwise, if all of the bits in 
    that block are zeros, then the bit's value is zero.

    https://imgur.com/a/NYLXp8m
*/
pub struct HierarchicalBitVec {
    levels: Vec<BitVec>
}

impl HierarchicalBitVec {
    pub fn new() -> Self {
        return Self {
            levels: Vec::new()
        }
    }
    
    pub fn with_bits(num_bits: usize, value_of_bits: bool) -> Self {
        if num_bits == 0 {
            return Self {
                levels: Vec::new()
            }
        }

        let mut levels: Vec<BitVec> = Vec::new();
        let mut num_bits_per_bit_at_level: usize = 1;
        loop {
            let num_bits_needed_at_level: usize = ((num_bits-1) / num_bits_per_bit_at_level) + 1;
            levels.push( BitVec::with_bits(num_bits_needed_at_level, value_of_bits) );
            if num_bits_needed_at_level <= BITS_PER_BLOCK {
                break;
            }
            num_bits_per_bit_at_level *= BITS_PER_BLOCK;
        }
    
        return Self{ levels }
    }

    pub fn num_bits(&self) -> usize {
        if self.levels.is_empty() {
            return 0
        }

        return self.levels[0].num_bits()
    }

    pub fn get_bit(&self, idx: usize) -> bool {
        return self.levels[0].get_bit(idx)
    }

    pub fn set_bit(&mut self, idx: usize, value: bool) {
        self.levels[0].set_bit(idx, value);
        assert!(self.levels[0].get_bit(idx) == value);

        let mut level: usize = 1;
        let mut idx_of_parent_bit: usize = idx / BITS_PER_BLOCK;
        while level < self.levels.len() {
            let idx_of_child_flags: usize = idx_of_parent_bit;
            let children: Block = self.levels[level-1].get_block(idx_of_child_flags);
            let value_of_parent_bit: bool = children != 0;
            self.levels[level].set_bit(idx_of_parent_bit, value_of_parent_bit);

            level += 1;
            idx_of_parent_bit = idx_of_parent_bit / BITS_PER_BLOCK;
        }
    }

    pub fn add_bits(&mut self, num_bits: usize, value: bool) {
        if num_bits == 0 {
            return
        }

        if self.levels.is_empty() {
            *self = Self::with_bits(num_bits, value);
            return
        }

        let new_num_bits: usize = self.levels[0].num_bits() + num_bits;
        let num_bits_each_level_encompasses: usize = self.levels[0].num_bits();
        let idx_of_first_new_bit_at_level_0: usize = num_bits_each_level_encompasses;

        let mut level: usize = 0;
        let mut num_bits_per_bit_at_level: usize = 1;
        while level < self.levels.len() {
            let num_bits_at_level: usize = self.levels[level].num_bits;
            let max_num_bits_level_can_encompass: usize = num_bits_at_level * num_bits_per_bit_at_level;
            assert!(num_bits_each_level_encompasses <= max_num_bits_level_can_encompass);
            let num_bits_level_can_encompass: usize = max_num_bits_level_can_encompass - num_bits_each_level_encompasses;
            if num_bits_level_can_encompass >= num_bits {
                break;
            }

            let num_bits_to_encompass: usize = num_bits - num_bits_level_can_encompass;
            let num_bits_for_level: usize = ((num_bits_to_encompass-1)/num_bits_per_bit_at_level)+1;
            self.levels[level].add_bits(num_bits_for_level, value);
            assert!( (self.levels[level].num_bits() * num_bits_per_bit_at_level) >= new_num_bits );
            
            level += 1;
            num_bits_per_bit_at_level *= BITS_PER_BLOCK;
        }

        if value == true {
            let mut level: usize = 0;
            let mut idx_of_bit: usize = idx_of_first_new_bit_at_level_0;
            while level < self.levels.len() {
                self.levels[level].set_bit_and_all_bits_after_it_to_true(idx_of_bit);
                level += 1;
                idx_of_bit /= BITS_PER_BLOCK;
            }
        }

        let top_level: &BitVec = &self.levels[self.levels.len()-1];
        if top_level.num_blocks() > 1 {
            let mut num_bits_per_bit_at_level: usize = 1;
            for _ in 0..self.levels.len() {
                num_bits_per_bit_at_level *= BITS_PER_BLOCK;
            }

            let mut level = self.levels.len();
            loop {
                let num_bits_needed_to_fit_items: usize = ((new_num_bits-1) / num_bits_per_bit_at_level) + 1;
                assert!(num_bits_needed_to_fit_items > 0);
                let mut new_level: BitVec = BitVec::with_bits(num_bits_needed_to_fit_items, value);
                for idx_of_parent_bit in 0..num_bits_needed_to_fit_items {
                    let idx_of_child_flags: usize = idx_of_parent_bit;
                    let do_child_flags_have_a_one: bool = self.levels[level-1].get_block(idx_of_child_flags) != 0;
                    new_level.set_bit(idx_of_parent_bit, do_child_flags_have_a_one);
                }
                self.levels.push(new_level);
                
                if num_bits_needed_to_fit_items <= BITS_PER_BLOCK {
                    break;
                }

                level += 1;
                num_bits_per_bit_at_level *= BITS_PER_BLOCK;
            }
        }
        assert!(self.levels[self.levels.len()-1].num_blocks() == 1);
    }

    pub fn find_a_true_bit(&self) -> Option<usize> {
        if self.levels.is_empty() {
            return None
        }

        let top_level: &BitVec = &self.levels[ self.levels.len()-1 ];
        assert!(top_level.flags.len() == 1);
        if top_level.get_block(0) == 0 {
            return None
        }

        let mut level: usize = self.levels.len()-1;
        let mut idx_of_parent_bit: usize = top_level.get_block(0).trailing_zeros() as usize; 
        loop {
            if level <= 0 {
                return Some( idx_of_parent_bit )
            }

            let idx_of_child_flags: usize = idx_of_parent_bit;
            level -= 1;
            idx_of_parent_bit =
                idx_of_child_flags * BITS_PER_BLOCK + 
                self.levels[level].get_block(idx_of_child_flags).trailing_zeros() as usize;
        }
    }

    /*
        todo: find_nearest_true_bit(&self, bit: usize) -> Option<usize>

        From bit, look to see if any of the bits in its block are open. If there is at
        least one open then walk two indices, initialized to bit, out to the left and 
        right of bit, looking for the closest open bit. If no bits in its block are open, 
        go to the block above it and repeat the same thing with the bit/BITS_PER_BLOCK 
        bit on the level above. This is the "going up" part.
        
        If you find an open bit in the block, then transition to the "going down" part. 
        If the bit was on the left side of the current level bit, then descend to that
        bit's block and look for the rightmost possible bits in each block at each level
        until you hit level 0. If the open bit was on the right side of the current level
        bit, then descend down and look for the leftmost possible bit at each level, until
        you hit level 0. If you go all the way up to the topmost level and there are no open
        bits, return None.

        This function would allow a hierarchical pool to efficiently place allocations
        as close to each other as possible, like this:

            // Allocate thing1, thing2, and thing3 as close to each other as possible
            group_of_things = [thing1, thing2, thing3]
            thing1_bit = pool.allocate(thing1)
            thing2_bit = pool.allocate_with_hint(thing1_bit)
            thing3_bit = pool.allocate_with_hint(thing1_bit)
            
            // Allocate as close to the start of the pool as possible
            group_of_things = [thing1, thing2, thing3]
            thing1_bit = pool.allocate_with_hint(0)
            thing2_bit = pool.allocate_with_hint(0)
            thing3_bit = pool.allocate_with_hint(0)
    */

    pub fn true_bits(&self) -> TrueBitsIterator {
        return TrueBitsIterator::new(self)
    }
}

pub struct TrueBitsIterator<'a> {
    levels: &'a Vec<BitVec>,
    stack: Vec<(usize, usize)>,
    flags: Block,
    base_global_idx_of_flags: usize,
}

impl <'a> TrueBitsIterator<'a> {
    fn new(bits: &'a HierarchicalBitVec) -> Self {
        let mut stack: Vec<(usize, usize)> = Vec::new();
        if !bits.levels.is_empty() && bits.levels.last().unwrap().get_block(0) != 0 {
            stack.push( (bits.levels.len()-1, 0) );
        }

        return Self {
            levels: &bits.levels,
            stack,
            flags: 0,
            base_global_idx_of_flags: 0,
        }
    }
}

impl <'a> Iterator for TrueBitsIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.flags == 0 {
            if self.stack.is_empty() {
                return None
            }

            while !self.stack.is_empty() {
                let (level, idx_of_flags): (usize, usize) = self.stack.pop().unwrap();
                if level == 0 {
                    self.flags = self.levels[0].get_block(idx_of_flags);
                    self.base_global_idx_of_flags = idx_of_flags * BITS_PER_BLOCK;
                    break;
                }
    
                let flags: Block = self.levels[level].get_block(idx_of_flags);
                assert!(flags != 0); // flag has at least one true bit
                let base_idx: usize = idx_of_flags * BITS_PER_BLOCK;

                // rev() because we're pushing to a stack and want lower bits to be the first
                // ones iterated over, traversing left to right
                for bit in (0..BITS_PER_BLOCK).rev() {
                    if (flags & (1 << bit)) != 0 {
                        let level: usize = level-1;
                        let idx_of_child_flags: usize = base_idx + bit;
                        self.stack.push( (level, idx_of_child_flags) );
                    }
                }
            }
        }
        
        assert!(self.flags != 0);
        let flags_idx_of_next_true_bit: usize = self.flags.trailing_zeros() as usize;
        self.flags &= !(1 << flags_idx_of_next_true_bit);
        let global_idx_of_next_true_bit: usize = self.base_global_idx_of_flags + flags_idx_of_next_true_bit;
        return Some(global_idx_of_next_true_bit)
    }
}

impl FlagVec for HierarchicalBitVec {
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

impl HierarchicalBitVec {
    /*
        This was carried over from another project where both BitVec and HierarchicalBitVec 
        implemented a get_error() function, each with their own InternalStateErrors.
    */
    fn _get_error(&self) -> Result<(), _InternalStateError> {
        use _InternalStateError::*;
        
        if self.levels.len() == 0 {
            return Ok(())
        }

        // Call get_error() on each BitVec in self.levels

        if !( self.levels[self.levels.len()-1].num_blocks() == 1 ) {
            return Err( TopmostLevelHasMoreThanOneFlag )
        }

        if !( self.levels[0].num_bits() > 0 ) {
            return Err( LevelsIsAllocatedButLevelZeroHasZeroBits )
        }

        let num_bits: usize = self.levels[0].num_bits();

        let mut num_bits_per_bit_at_level: usize = 1;
        for level in 0..self.levels.len() {
            let max_num_bits_level_can_encompass: usize = self.levels[level].num_bits() * num_bits_per_bit_at_level;
            if !( num_bits <= max_num_bits_level_can_encompass ) {
                return Err(
                    LevelContainsLessBitsThanLevelZero {
                        level,
                        max_num_bits_level_can_encompass,
                        num_bits_level_zero_encompasses: self.levels[0].num_bits(), 
                    }
                )
            }
            
            let min_num_bits_needed_to_fit_num_bits_at_level: usize = ((num_bits-1)/num_bits_per_bit_at_level)+1;
            if !( self.levels[level].num_bits() == min_num_bits_needed_to_fit_num_bits_at_level ) {
                return Err(
                    LevelEncompassesMoreBitsThanAreNeededToFitNumBits {
                        level, 
                        min_num_bits_needed_to_fit_num_bits_at_level,
                        num_bits_level_zero_encompasses: self.levels[0].num_bits(),
                    }
                )
            }

            num_bits_per_bit_at_level *= BITS_PER_BLOCK;
        }

        let mut level: usize = 1;
        while level < self.levels.len() {
            for idx_of_parent_bit in 0..self.levels[level].num_bits() {
                let idx_of_child_flags: usize = idx_of_parent_bit;
                let child_flags: Block = self.levels[level-1].get_block(idx_of_child_flags);
                let parent_bit: Block = self.levels[level].get_bit(idx_of_parent_bit) as Block;
                if !( (parent_bit == 0) == (child_flags == 0) ) {
                    return Err(
                        ParentBitDoesntMatchChildBlock {
                            idx_of_parent_bit, parent_bit, parent_level: level,
                            idx_of_child_flags, child_flags, child_level: level-1,
                        }
                    )
                }
            }
            level += 1;
        }

        return Ok(())
    }
}

#[derive(Clone, Copy, Debug)]
enum _InternalStateError {
    LevelsIsAllocatedButLevelZeroHasZeroBits,

    TopmostLevelHasMoreThanOneFlag,

    LevelContainsLessBitsThanLevelZero {
        level: usize, 
        max_num_bits_level_can_encompass: usize,
        num_bits_level_zero_encompasses: usize, 
    },
    
    LevelEncompassesMoreBitsThanAreNeededToFitNumBits {
        level: usize, 
        min_num_bits_needed_to_fit_num_bits_at_level: usize,
        num_bits_level_zero_encompasses: usize,
    },

    ParentBitDoesntMatchChildBlock {
        idx_of_parent_bit: usize,
        parent_bit: Block, 
        parent_level: usize,
        idx_of_child_flags: usize, 
        child_flags: Block, 
        child_level: usize,
    },
}