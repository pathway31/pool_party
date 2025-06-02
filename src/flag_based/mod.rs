mod bit;
mod bool;
mod hierarchical;

use crate::Pool;

#[allow(dead_code)]
pub type BitFlags<T> = FlagsBasedPool<T, bit::BitVec>;

#[allow(dead_code)]
pub type BoolFlags<T> = FlagsBasedPool<T, bool::BoolVec>;

#[allow(dead_code)]
pub type HierarchicalFlags<T> = FlagsBasedPool<T, hierarchical::HierarchicalBitVec>;

pub trait FlagVec {
    type TrueFlagsIter<'a>: Iterator<Item=usize> where Self: 'a;

    fn new() -> Self;
    fn with_flags(num_flags: usize, value: bool) -> Self;
    fn num_flags(&self) -> usize;
    fn get_flag(&self, flag: usize) -> bool;
    fn set_flag(&mut self, flag: usize, value: bool);
    fn add_flags(&mut self, num_flags: usize, value: bool);
    fn find_a_true_flag(&self) -> Option<usize>;
    fn true_flags<'a>(&'a self) -> Self::TrueFlagsIter<'a>;
}

pub struct FlagsBasedPool<T: Clone, U: FlagVec> {
    alloc: U, // flags indicating an item is allocated (0 for deallocated, 1 for allocated)
    free: U, // flags indicating an item is deallocated (0 for deallocated, 1 for allocated)
    items: Vec<Option<T>>,
    num_items: usize,
}

impl <T: Clone, U: FlagVec> Pool<T> for FlagsBasedPool<T, U> {
    type Iter<'a> = Iter<'a, T, U> where Self: 'a, T: 'a;

    fn new() -> Self {
        return Self {
            alloc: FlagVec::new(),
            free: FlagVec::new(),
            items: Vec::new(),
            num_items: 0,
        }
    }

    fn with_capacity(num_items: usize) -> Self {
        return Self {
            alloc: FlagVec::with_flags(num_items, false),
            free: FlagVec::with_flags(num_items, true),
            items: vec![None; num_items],
            num_items: 0,
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

        let id: usize = self.free.find_a_true_flag().unwrap();
        assert!(self.alloc.get_flag(id) == false);
        assert!(self.free.get_flag(id) == true);

        self.alloc.set_flag(id, true);
        self.free.set_flag(id, false);
        assert!(self.items[id].is_none());
        self.items[id] = Some(item);
        self.num_items += 1;
        return id
    }

    fn deallocate(&mut self, id: usize) {
        assert!(self.items[id].is_some());
        assert!(self.alloc.get_flag(id) == true);
        assert!(self.free.get_flag(id) == false);

        self.alloc.set_flag(id, false);
        self.free.set_flag(id, true);
        self.items[id] = None; // calls Drop on the item
        self.num_items -= 1;
    }

    fn iter<'a>(&'a self) -> Iter<'a, T, U> {
        return Iter::new(&self.items, &self.alloc)
    }
}

impl <T: Clone, U: FlagVec> FlagsBasedPool<T, U> {
    fn expand_if_needed(&mut self) {
        if self.num_items < self.alloc.num_flags() {
            return
        }
        
        const GROWTH_FACTOR: usize = 2;
        let new_num_items: usize =
            if self.num_items == 0 {
                1
            }
            else {
                self.num_items*GROWTH_FACTOR
            };
        let num_new_items: usize = new_num_items - self.num_items;
        self.alloc.add_flags(num_new_items, false);
        self.free.add_flags(num_new_items, true);
        self.items.resize(new_num_items, None);
    }
}

pub struct Iter<'a, T: Clone, U: 'a + FlagVec> {
    items: &'a Vec<Option<T>>,
    true_flags_iter: <U as FlagVec>::TrueFlagsIter<'a>,
}

impl <'a, T: Clone, U: FlagVec> Iter<'a, T, U> {
    fn new(items: &'a Vec<Option<T>>, alloc: &'a U) -> Self {
        return Self { items, true_flags_iter: alloc.true_flags() }
    }
}

impl <'a, T: 'a + Clone, U: 'a + FlagVec> Iterator for Iter<'a, T, U> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        match self.true_flags_iter.next() {
            Some(id) => return Some( self.items[id].as_ref().unwrap() ),
            None => return None
        }
    }
}

#[cfg(test)]
mod tests {
    mod bool {
        use super::super::BoolFlags;
        use crate::testing;
        use crate::testing::Item; 
    
        type Pool = BoolFlags<Item>;
    
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

    mod bit {
        use super::super::BitFlags;
        use crate::testing;
        use crate::testing::Item; 
    
        type Pool = BitFlags<Item>;
    
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

    mod hierarchical {
        use super::super::HierarchicalFlags;
        use crate::testing;
        use crate::testing::Item; 
    
        type Pool = HierarchicalFlags<Item>;
    
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
}