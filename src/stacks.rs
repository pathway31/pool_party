use std::slice;
use std::mem::size_of;
use super::Pool;

type Block = u8;
const BITS_PER_BYTE: usize = 8;
const FLAGS_PER_BLOCK: usize = size_of::<Block>() * BITS_PER_BYTE;
const EMPTY_BLOCK: Block = 0;
const FULL_BLOCK: Block = Block::MAX;

pub struct Stacks<T: Clone> {
    items: Vec<Option<T>>,
    num_items: usize,

    flags: Vec<Block>, // flags for each item (0 for unallocated, 1 for allocated)
    open_blocks: Vec<usize>,  // indices of blocks that have at least one item unallocated
    alloc_blocks: Vec<usize>, // indices of blocks that have one or more items allocated
}

impl <T: Clone> Pool<T> for Stacks<T> {
    type Iter<'a> = Iter<'a, T> where Self: 'a, T: 'a;
    
    fn new() -> Self {
        return Self {
            items: Vec::new(),
            num_items: 0,
            
            flags: Vec::new(),
            open_blocks: Vec::new(),
            alloc_blocks: Vec::new(),
        }        
    }

    fn with_capacity(num_items: usize) -> Self {
        let num_blocks: usize;
        if num_items == 0 {
            num_blocks = 0;
        }
        else {
            num_blocks = ((num_items-1)/FLAGS_PER_BLOCK)+1;
        };
        
        let items: Vec<Option<T>> = vec![None; num_blocks*FLAGS_PER_BLOCK];
        let num_items: usize = 0;

        let flags: Vec<Block> = vec![0; num_blocks];
        let open_blocks: Vec<usize> = (0..num_blocks).rev().collect();
        let alloc_blocks: Vec<usize> = Vec::new();

        return Self {
            items,
            num_items,

            flags,
            open_blocks,
            alloc_blocks,
        }
    }

    fn len(&self) -> usize {
        return self.num_items
    }

    fn get(&self, id: usize) -> &T {
        return self.items[id].as_ref().unwrap()
    }

    fn get_mut(&mut self, id: usize) -> &mut T {
        return self.items[id].as_mut().unwrap()
    }

    fn allocate(&mut self, item: T) -> usize {
        self.expand_if_needed();

        let open_block: usize = *self.open_blocks.last().unwrap();
        assert!(self.flags[open_block] != FULL_BLOCK);
        let local_bit: usize = self.flags[open_block].trailing_ones() as usize;
        self.flags[open_block] |= 1 << local_bit;

        let block_is_now_full: bool = self.flags[open_block] == FULL_BLOCK;
        if block_is_now_full {
            self.open_blocks.pop().unwrap();
        }

        let block_now_has_one_item_allocated: bool = self.flags[open_block].count_ones() == 1;
        if block_now_has_one_item_allocated {
            self.alloc_blocks.push(open_block);
        }

        let global_bit: usize = open_block*FLAGS_PER_BLOCK + local_bit;
        assert!(self.items[global_bit].is_none());
        self.items[global_bit] = Some(item);
        self.num_items += 1;

        return global_bit
    }

    fn deallocate(&mut self, id: usize) {
        assert!(self.items[id].is_some());
        let block: usize = id / FLAGS_PER_BLOCK;
        let local_bit: usize = id % FLAGS_PER_BLOCK;
        self.flags[block] &= !(1 << local_bit);

        if self.flags[block] == EMPTY_BLOCK {
            for i in 0..self.alloc_blocks.len() {
                if self.alloc_blocks[i] == block {
                    self.alloc_blocks.remove(i); // O(n)
                    break;
                }
            }
        }

        if self.flags[block].count_ones() as usize == FLAGS_PER_BLOCK-1 {
            self.open_blocks.push(block);
        }

        let global_bit: usize = block*FLAGS_PER_BLOCK + local_bit;
        assert!(self.items[global_bit].is_some());
        self.items[global_bit] = None;

        self.num_items -= 1;
    }

    fn iter<'a>(&'a self) -> Iter<'a, T> {
        return Iter::new(
            &self.items,
            &self.flags,
            &self.alloc_blocks,
        )
    }
}

impl <T: Clone> Stacks<T> {
    fn expand_if_needed(&mut self) {
        if !self.open_blocks.is_empty() {
            return
        }

        assert!(self.alloc_blocks.len() == self.flags.len());
        assert!(self.flags.iter().all(|block: &Block| *block == FULL_BLOCK));

        const GROWTH_FACTOR: usize = 2;
        let old_num_blocks: usize = self.flags.len();
        let new_num_blocks: usize;
        if self.num_items == 0 {
            assert!(old_num_blocks == 0);
            new_num_blocks = 1;
        }
        else {
            new_num_blocks = old_num_blocks*GROWTH_FACTOR;
        };
        let new_num_items: usize = new_num_blocks * FLAGS_PER_BLOCK;
        
        self.items.resize(new_num_items, None);
        self.flags.resize(new_num_blocks, 0);
        self.open_blocks.append( &mut (old_num_blocks..new_num_blocks).rev().collect() ); 
        assert!(self.items.len() == self.flags.len()*FLAGS_PER_BLOCK);
    }
}

pub struct Iter<'a, T: Clone> {
    items: &'a Vec<Option<T>>,
    flags: &'a Vec<Block>,
    block: Block,
    offset: usize,
    alloc_blocks: slice::Iter<'a, usize>,
}

impl <'a, T: Clone> Iter<'a, T> {
    fn new(items: &'a Vec<Option<T>>, flags: &'a Vec<Block>, alloc_blocks: &'a Vec<usize>) -> Self {
        return Self { 
            items,
            flags,
            block: 0,
            offset: 0,
            alloc_blocks: alloc_blocks.iter()
        }
    }
}

impl <'a, T: Clone> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.block == EMPTY_BLOCK {
            match self.alloc_blocks.next() {
                Some(block) => {
                    self.block = self.flags[*block];
                    self.offset = (*block)*FLAGS_PER_BLOCK;
                },

                None => return None,
            }
        }

        assert!(self.block != 0);
        let local_bit: usize = self.block.trailing_zeros() as usize;
        self.block &= !(1 << local_bit);
        let global_bit: usize = self.offset + local_bit;
        return Some(self.items[global_bit].as_ref().unwrap())
    }
}

#[cfg(test)]
mod tests {
    use super::Stacks;
    use crate::testing;
    use crate::testing::Item; 

    type Pool = Stacks<Item>;

    #[test]
    #[should_panic]
    fn test_invalid_get_to_empty_pool() {
        testing::test_invalid_get_to_empty_pool::<Pool>();
    }

    #[test]
    #[should_panic]
    fn test_invalid_get_to_nonempty_pool() {
        testing::test_invalid_get_to_nonempty_pool::<Pool>();
    }

    #[test]
    fn test_one_item() {
        testing::test_one_item::<Pool>();
    }

    #[test]
    fn test_many_items() {
        testing::test_many_items::<Pool>();
    }

    #[test]
    fn fuzz_many_pools_few_mutations() {
        testing::fuzz_many_pools_few_mutations::<Pool>();
    }

    #[test]
    fn fuzz_few_pools_many_mutations() {
        testing::fuzz_few_pools_many_mutations::<Pool>();
    }
}