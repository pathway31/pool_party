use crate::Pool;
use std::collections::HashMap;
use std::collections::hash_map::Values;

pub struct Reference<T> {
    map: HashMap<usize, Box<T>>,
}

impl <T> Pool<T> for Reference<T> {
    type Iter<'a> = Iter<'a, T> where Self: 'a, T: 'a;

    fn new() -> Self {
        return Self {
            map: HashMap::default()
        }
    }

    fn with_capacity(num_items: usize) -> Self {
        return Self {
            map: HashMap::with_capacity(num_items)
        }
    }

    fn len(&self) -> usize {
        return self.map.len()
    }

    fn get(&self, id: usize) -> &T {
        let item: &Box<T> = self.map.get(&id).unwrap();
        return item.as_ref()
    }

    fn get_mut(&mut self, id: usize) -> &mut T {
        let item: &mut Box<T> = self.map.get_mut(&id).unwrap();
        return item.as_mut()
    }

    fn allocate(&mut self, item: T) -> usize {
        let item: Box<T> = Box::new(item);
        let address: usize = (&(*item) as *const T) as usize;
        self.map.insert(address, item);
        return address
    }

    fn deallocate(&mut self, id: usize) {
        assert!(self.map.get(&id).is_some());
        self.map.remove(&id);
    }

    fn iter<'a>(&'a self) -> Self::Iter<'a> {
        return Iter::new( self.map.values() )
    }
}

pub struct Iter<'a, T> {
    inner: Values<'a, usize, Box<T>>
}

impl <'a, T> Iter<'a, T> {
    fn new(inner: Values<'a, usize, Box<T>>) -> Self {
        return Self { inner }
    }
}

impl <'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;
    
    fn next(&mut self) -> Option<Self::Item> {
        match self.inner.next() {
            Some(next) => return Some(next.as_ref()),
            None       => return None,
        }
    }
}