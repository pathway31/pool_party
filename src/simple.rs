use std::slice;

use super::Pool;

pub struct Simple<T: Clone> {
    items: Vec<Option<T>>,
    num_items: usize,
}

impl <T: Clone> Pool<T> for Simple<T> {
    type Iter<'a> = Iter<'a, T> where Self: 'a, T: 'a;

    fn new() -> Self {
        return Self{ 
            items: Vec::new(),
            num_items: 0,
        }
    }

    fn with_capacity(num_items: usize) -> Self {
        return Self { 
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
        const GROWTH_FACTOR: usize = 2;
        if self.num_items == self.items.len() {
            let new_num_items: usize = if self.num_items == 0 { 1 } else { self.num_items*GROWTH_FACTOR };
            self.items.resize(new_num_items, None);
        }

        for id in 0..self.items.len() {
            if self.items[id].is_none() {
                self.items[id] = Some(item);
                self.num_items += 1;
                return id
            }
        }
        unreachable!();
    }

    fn deallocate(&mut self, id: usize) {
        assert!(self.items[id].is_some());
        self.items[id] = None;
        self.num_items -= 1;
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        return Iter::new(self.items.iter())
    }
}

pub struct Iter<'a, T> {
    inner: slice::Iter<'a, Option<T>>
}

impl <'a, T> Iter<'a, T> {
    fn new(inner: slice::Iter<'a, Option<T>>) -> Self {
        return Self { inner }
    }
}

impl <'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inner.next() {
                Some(item) => {
                    if let Some(item) = item {
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
    use super::Simple;
    use crate::testing;
    use crate::testing::Item; 

    type Pool = Simple<Item>;

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