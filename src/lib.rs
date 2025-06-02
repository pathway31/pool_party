mod reference;
mod simple;
mod stacks;
mod freelist;
mod notsafe;
mod flag_based;

#[cfg(test)]
mod testing;

pub trait Pool<T> {
    type Iter<'a>: Iterator<Item=&'a T> where Self: 'a, T: 'a;

    fn new() -> Self;
    fn with_capacity(num_items: usize) -> Self;
    fn len(&self) -> usize;
    fn get(&self, id: usize) -> &T;
    fn get_mut(&mut self, id: usize) -> &mut T;
    fn allocate(&mut self, item: T) -> usize;
    fn deallocate(&mut self, id: usize);
    fn iter<'a>(&'a self) -> Self::Iter<'a>;
}