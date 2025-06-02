use super::Pool;
use std::ptr::null;
use std::ptr::null_mut;
use std::mem::size_of;

type FlagBlock = u8;
const BITS_PER_BYTE: usize = 8;
const FLAGS_PER_BLOCK: usize = size_of::<FlagBlock>() * BITS_PER_BYTE;
const EMPTY_BLOCK: FlagBlock = 0;
const FULL_BLOCK: FlagBlock = FlagBlock::MAX;

struct Node {
    block: usize, // index of block which has at least one item allocated in it
    prev: *mut Node,
    next: *mut Node,
}

pub struct NotSafe<T: Clone> {
    items: Vec<Option<T>>,
    num_items: usize,

    flags: Vec<FlagBlock>, // item allocation flags for each block (0 for unallocated, 1 for allocated)
    open_blocks: Vec<usize>, // stack containing indices of blocks which contain at least one unallocated item
    nodes: Vec<*mut Node>, // map from a block's index to its entry in the linked list
    head: *mut Node, // linked list of blocks which have at least one item allocated
}

impl <T: Clone> Pool<T> for NotSafe<T> {
    type Iter<'a> = Iter<'a, T> where Self: 'a, T: 'a;
    
    fn new() -> Self {
        return Self {
            items: Vec::new(),
            num_items: 0,

            flags: Vec::new(),
            open_blocks: Vec::new(),
            nodes: Vec::new(),
            head: null_mut(),
        }
    }

    fn with_capacity(capacity: usize) -> Self {
        let num_blocks: usize = 
            if capacity == 0 {
                0                                
            } 
            else {
                ((capacity-1)/FLAGS_PER_BLOCK)+1
            };
    
        let items: Vec<Option<T>> = vec![None; num_blocks*FLAGS_PER_BLOCK];
        let num_items: usize = 0;

        let flags: Vec<FlagBlock> = vec![0; num_blocks];
        let open_blocks: Vec<usize> = (0..num_blocks).rev().collect();
        let nodes: Vec<*mut Node> = vec![null_mut(); num_blocks];
        let head: *mut Node = null_mut();

        return Self {
            items,
            num_items,

            flags,
            open_blocks,
            nodes,
            head,
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
            let node: Node = Node { 
                block: open_block,
                prev: null_mut(),
                next: self.head,
            };
            let node: *mut Node = Box::into_raw(Box::new(node));
    
            assert!(self.nodes[open_block] == null_mut());
            self.nodes[open_block] = node;
    
            unsafe {
                if self.head != null_mut() {
                    assert!((*self.head).prev == null_mut());
                    (*self.head).prev = node;
                }
                self.head = node;
            }
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
        assert!( ((self.flags[block] & (1 << local_bit)) >> local_bit) == 1);
        let was_block_full: bool = self.flags[block] == FULL_BLOCK;
        self.flags[block] &= !(1 << local_bit); // zero out the flag

        let block_is_no_longer_full: bool = was_block_full;
        if block_is_no_longer_full {
            self.open_blocks.push(block);
        }

        unsafe {
            let block_is_now_empty: bool = self.flags[block] == EMPTY_BLOCK;
            if block_is_now_empty {
                assert!(self.nodes[block] != null_mut());
                let node: *mut Node = self.nodes[block];
                if node == self.head {
                    self.head = (*node).next;
                }
                if (*node).prev != null_mut() {
                    (*(*node).prev).next = (*node).next;
                }
                if (*node).next != null_mut() {
                    (*(*node).next).prev = (*node).prev;
                }
                let _drop: Box<Node> = Box::from_raw(node);
                self.nodes[block] = null_mut();
                // _drop goes out of scope and is dropped
            }
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
            self.head as *const Node,
        )
    }
}

impl <T: Clone> NotSafe<T> {
    fn expand_if_needed(&mut self) {
        if !self.open_blocks.is_empty() {
            return
        }

        assert!(self.flags.iter().all(|block: &FlagBlock| *block == FULL_BLOCK), "{:?}", self.flags);
        assert!(self.nodes.len() == self.flags.len());

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
        self.flags.resize(new_num_blocks, EMPTY_BLOCK);
        self.open_blocks.append( &mut (old_num_blocks..new_num_blocks).rev().collect() );
        self.nodes.resize(new_num_blocks, null_mut());
        assert!(self.items.len() == self.flags.len()*FLAGS_PER_BLOCK);
    }
}

impl <T: Clone> Drop for NotSafe<T> {
    fn drop(&mut self) {
        let mut curr: *mut Node = self.head;
        while curr != null_mut() {
            let next: *mut Node = unsafe{ (*curr).next };
            let _drop: Box<Node> = unsafe{ Box::from_raw(curr) };
            curr = next;
            // _drop goes out of scope and is dropped
        }
    }
}

pub struct Iter<'a, T: Clone> {
    items: &'a Vec<Option<T>>,
    flags: &'a Vec<FlagBlock>,
    next_node: *const Node,
    curr_flags: FlagBlock,
    curr_offset: usize, // self.curr_flags*FLAGS_PER_BLOCK
}

impl <'a, T: Clone> Iter<'a, T> {
    fn new(
        items: &'a Vec<Option<T>>, 
        flags: &'a Vec<FlagBlock>, 
        head: *const Node
    ) -> Self {
        if head == null() {
            return Self {
                items,
                flags,
                next_node: null_mut(),
                curr_flags: 0,
                curr_offset: 0
            }
        }

        unsafe {
            return Self {
                items,
                flags,
                next_node: (*head).next,
                curr_flags: flags[(*head).block],
                curr_offset: (*head).block * FLAGS_PER_BLOCK
            }
        }
    }
}

impl <'a, T: Clone> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            if self.curr_flags == EMPTY_BLOCK {
                if self.next_node == null() {
                    return None
                }

                let curr: *const Node = self.next_node;
                self.next_node = (*curr).next;
                self.curr_flags = self.flags[(*curr).block];
                assert!(self.curr_flags != 0);
                self.curr_offset = (*curr).block * FLAGS_PER_BLOCK;
            }

            let local_offset: FlagBlock = self.curr_flags.trailing_zeros() as FlagBlock;
            let global_offset: usize = self.curr_offset + local_offset as usize;
            self.curr_flags &= !(1 << local_offset); // consume the flag
            let item: &T = self.items[global_offset].as_ref().unwrap();
            return Some(item)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NotSafe;
    use crate::testing;
    use crate::testing::Item; 

    type Pool = NotSafe<Item>;

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