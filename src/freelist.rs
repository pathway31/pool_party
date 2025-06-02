use std::slice;
use super::Pool;

#[derive(Clone)]
enum Slot<T> {
    Item(T),
    Free{next_free_slot: Option<usize>},
}

pub struct FreeList<T: Clone> {
    slots: Vec<Slot<T>>,
    next_free_slot: Option<usize>,
    num_items: usize,
}

impl <T: Clone> Pool<T> for FreeList<T> {
    type Iter<'a> = Iter<'a, T> where Self: 'a, T: 'a;

    fn new() -> Self {
        return Self { 
            slots: Vec::new(),
            next_free_slot: None,
            num_items: 0,
        }
    }
    
    fn with_capacity(num_items: usize) -> Self {
        if num_items == 0 {
            return Self {
                slots: Vec::new(),
                next_free_slot: None,
                num_items: 0,
            }
        }
    
        let mut slots: Vec<Slot<T>> = vec![Slot::Free{ next_free_slot: None }; num_items];
        for i in 1..num_items {
            slots[i-1] = Slot::Free{ next_free_slot: Some(i) };
        }
        slots[num_items-1] = Slot::Free{ next_free_slot: None };
        
        return Self {
            slots,
            next_free_slot: Some(0),
            num_items: 0,
        }
    }
    
    fn len(&self) -> usize {
        return self.num_items
    }

    fn get(&self, id: usize) -> &T {
        match &self.slots[id] {
            Slot::Item(item) => return item,
            Slot::Free{..} => panic!(),
        }
    }

    fn get_mut(&mut self, id: usize) -> &mut T {
        match &mut self.slots[id] {
            Slot::Item(item) => return item,
            Slot::Free{..} => panic!(),
        }
    }

    fn allocate(&mut self, item: T) -> usize {
        self.expand_if_needed();
        let free_slot_for_item: usize = self.next_free_slot.unwrap();
        match self.slots[free_slot_for_item] {
            Slot::Free{ next_free_slot } => {
                self.next_free_slot = next_free_slot;
            },
            
            Slot::Item(_) => panic!(),
        }
        self.slots[free_slot_for_item] = Slot::Item(item);
        self.num_items += 1;
        return free_slot_for_item
    }

    fn deallocate(&mut self, item_id: usize) {
        if let Slot::Free{..} = self.slots[item_id] {
            assert!(false);
        }
        self.slots[item_id] = Slot::Free{next_free_slot: self.next_free_slot}; // drops the item contained in the slot
        self.next_free_slot = Some(item_id);
        self.num_items -= 1;
    }
    
    fn iter<'a>(&'a self) -> Iter<'a, T> {
        return Iter::new(self.slots.iter())
    }
}
    
impl <T: Clone> FreeList<T> {
    fn expand_if_needed(&mut self) {
        if self.next_free_slot.is_some() {
            return
        }

        const GROWTH_FACTOR: usize = 2;
        let old_num_items: usize = self.slots.len();
        let new_num_items: usize = 
            if self.slots.len() == 0 {
                1
            }
            else {
                old_num_items*GROWTH_FACTOR
            };
        self.slots.resize(new_num_items, Slot::Free{next_free_slot: None});
        for i in old_num_items..(new_num_items-1) {
            self.slots[i] = Slot::Free{next_free_slot: Some(i+1)};
        }
        assert!(self.slots.len() > 0);
        let last: usize = self.slots.len()-1;
        self.slots[last] = Slot::Free{next_free_slot: None};
        self.next_free_slot = Some(old_num_items);
    }
}

pub struct Iter<'a, T> {
    inner: slice::Iter<'a, Slot<T>>
}

impl <'a, T> Iter<'a, T> {
    fn new(inner: slice::Iter<'a, Slot<T>>) -> Self {
        return Self { inner }
    }
}

impl <'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(slot) => {
                    if let Slot::Item(item) = slot {
                        return Some(item)
                    }
                },

                None => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::FreeList;
    use crate::testing;
    use crate::testing::Item; 

    type Pool = FreeList<Item>;

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