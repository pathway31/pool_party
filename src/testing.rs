use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use rand::Rng;
use rand::SeedableRng;
use rand_xoshiro::Xoshiro256StarStar;
use super::Pool;
use super::reference::Reference;

pub type Item = i32;

// Should panic
pub fn test_invalid_get_to_empty_pool<T: Pool<Item>>() {
    let pool: T = Pool::new();
    pool.get(3);
}

// Should panic
pub fn test_invalid_get_to_nonempty_pool<T: Pool<Item>>() {
    let mut pool: T = Pool::new();
    let mut ids: Vec<usize> = Vec::new();
    for _ in 0..128 {
        let id: usize = pool.allocate(0);
        ids.push(id);
    }

    pool.deallocate(ids[0]);
    pool.get(ids[0]);
}

pub fn test_one_item<T: Pool<Item>>() {
    let mut pool: T = Pool::new();

    let id: usize = pool.allocate(72);
    assert!(pool.len() == 1);
    assert!(pool.iter().cloned().collect::<Vec<Item>>() == vec![72]);

    pool.deallocate(id);
    assert!(pool.len() == 0);
    assert!(pool.iter().next().is_none());
}

pub fn test_many_items<T: Pool<Item>>() {
    let mut pool: T = Pool::new();
    let mut map: HashMap<usize, Item> = HashMap::new();
    for i in 0..3827 {
        let item: Item = i;
        let id: usize = pool.allocate(item);
        map.insert(id, item);
    }

    for id in map.clone().keys() {
        if id/2 % 3 == 0 {
            pool.deallocate(*id);
            map.remove(id);
        }
    }

    assert!(pool.len() == map.len());
    
    for id in map.keys() {
        assert!(pool.get(*id) == map.get(id).unwrap());
    }

    let mut pool_items: Vec<Item> = pool.iter().cloned().collect();
    let mut map_items: Vec<Item> = map.values().cloned().collect();
    pool_items.sort();
    map_items.sort();
    assert!(pool_items == map_items);
}

pub fn fuzz_many_pools_few_mutations<T: Pool<Item>>() {
    const NUM_POOLS_TO_FUZZ: usize = 10_000;
    const MAX_NUM_MUTATIONS: usize = 10;
    fuzz_many_item_pools::<T>(NUM_POOLS_TO_FUZZ, MAX_NUM_MUTATIONS);
}

pub fn fuzz_few_pools_many_mutations<T: Pool<Item>>() {
    const NUM_POOLS_TO_FUZZ: usize = 100;
    const MAX_NUM_MUTATIONS: usize = 1000;
    fuzz_many_item_pools::<T>(NUM_POOLS_TO_FUZZ, MAX_NUM_MUTATIONS);
}

// Hey, it's like that Leetcode problem
// https://leetcode.com/problems/insert-delete-getrandom-o1/
#[derive(Clone, Copy, PartialEq, Eq)]
struct PairOfIds {
    id_in_test: usize,
    id_in_reference: usize,
}

impl Hash for PairOfIds {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.id_in_test);
        state.write_usize(self.id_in_reference);
    }
}

struct Allocations {
    map: HashMap<PairOfIds, usize>, // ids -> index in vec that contains ids
    vec: Vec<PairOfIds>,
}

impl Allocations {
    pub fn new() -> Self {
        return Self {
            map: HashMap::default(),
            vec: Vec::new(),
        }
    }

    pub fn add(&mut self, ids: PairOfIds) {
        self.map.insert(ids, self.vec.len());
        self.vec.push(ids);
    }

    // This function right here is the reason for this data structure to exist- it
    // removes ids in O(1) time. It's a lot faster to run tests with this vs something 
    // like a Vec with O(n) removal 
    pub fn remove(&mut self, removed_ids: PairOfIds) {        
        let index_of_removed_ids: usize = *self.map.get(&removed_ids).unwrap();
        let index_of_last_ids: usize = self.vec.len()-1;
        if index_of_removed_ids != index_of_last_ids {
            let last_ids: PairOfIds = self.vec[ index_of_last_ids ];
            self.vec[ index_of_removed_ids ] = last_ids;
            self.map.insert(last_ids, index_of_removed_ids);
        }
        self.vec.pop().unwrap();
        self.map.remove(&removed_ids);
    }

    pub fn get_random_pair_of_ids<T: Rng>(&self, rng: &mut T) -> Option<PairOfIds> {
        if self.vec.is_empty() {
            return None
        }

        let index: usize = rng.gen_range(0..self.len());
        return Some( self.vec[index] )
    }

    pub fn len(&self) -> usize {
        return self.map.len()
    }

    pub fn pairs_of_ids(&self) -> impl Iterator<Item=&PairOfIds> {
        return self.vec.iter()
    }
}

// Constructions
const NEW: usize = 0;
const WITH_CAPACITY: usize = 1;

#[derive(Clone, Copy, Debug)]
enum Construction {
    New,
    WithCapacity{capacity: usize},
}

fn generate_random_construction<T: Rng>(rng: &mut T) -> Construction {
    const MAX_CAPACITY: usize = 10_000;
    let construction: usize = rng.gen_range(NEW..=WITH_CAPACITY);
    match construction {
        NEW => {
            return Construction::New
        },

        WITH_CAPACITY => {
            let capacity: usize = rng.gen_range(0..MAX_CAPACITY);
            return Construction::WithCapacity{ capacity }
        },

        _ => unreachable!(),
    }
}

// Mutations
const SET: usize = 0;
const ALLOCATE: usize = 1;
const DEALLOCATE: usize = 2;
const NUM_MUTATIONS: usize = 3;

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum Mutation {
    Set{id_in_test: usize, id_in_reference: usize, value: Item},
    Allocate{item: Item},
    Deallocate{id_in_test: usize, id_in_reference: usize},
}

struct MutationGenerator {
    tokens: [usize; NUM_MUTATIONS],
    total_num_tokens: usize,
}

impl MutationGenerator {    
    const MAX_NUM_TOKENS: usize = 10;
    
    pub fn new<T: Rng>(rng: &mut T) -> Self {
        let mut tokens: [usize; NUM_MUTATIONS] = [0; NUM_MUTATIONS];
        let mut total_num_tokens: usize = 0;
        for mutation in 0..NUM_MUTATIONS {
            let num_tokens: usize = rng.gen_range(1..=Self::MAX_NUM_TOKENS);
            tokens[mutation] = num_tokens;
            total_num_tokens += num_tokens;
        }

        assert!(tokens.iter().cloned().all(|num_tokens| num_tokens > 0));

        return Self { 
            tokens,
            total_num_tokens 
        }
    }

    pub fn generate<T: Rng>(&mut self, rng: &mut T) -> usize {
        let token: usize = rng.gen_range(1..=self.total_num_tokens);
        let mut num_tokens_so_far: usize = 0;
        for mutation in 0..self.tokens.len() {
            assert!(token > num_tokens_so_far);
            let token: usize = token - num_tokens_so_far;
            if token <= self.tokens[mutation] {
                return mutation
            }
            num_tokens_so_far += self.tokens[mutation];
        }
        unreachable!();
    }

    pub fn _get_probability(&self, mutation: usize) -> f64 {
        return self.tokens[mutation] as f64 / self.total_num_tokens as f64
    }

    pub fn _relative_probabilities(&self) -> impl Iterator<Item=&usize> {
        return self.tokens.iter()
    }
}

#[derive(Debug)]
enum EqualityError {
    NumItemsDontMatch,
    ItemsDontMatch,
    GetsDontMatch,
}

fn compare_for_equality<T: Pool<Item>> (
    test: &T, 
    reference: &Reference<Item>,
    allocations: &Allocations,
) 
-> Result<(), EqualityError>
{
    if test.len() != reference.len() {
        return Err( EqualityError::NumItemsDontMatch )
    }

    let mut items_in_test: Vec<Item> = test.iter().copied().collect();
    let mut items_in_reference: Vec<Item> = reference.iter().copied().collect();
    items_in_test.sort();
    items_in_reference.sort();
    if items_in_test != items_in_reference {
        return Err( EqualityError::ItemsDontMatch )
    }

    for pair in allocations.pairs_of_ids() {
        if *test.get( pair.id_in_test ) != *reference.get( pair.id_in_reference ) {
            return Err( EqualityError::GetsDontMatch )
        }
    }

    return Ok(())
}

#[allow(dead_code)]
#[derive(Clone, Copy, Debug)]
enum LogEntry {
    Construction(Construction),
    Mutation(Mutation),
}

fn fuzz<T: Rng, U: Pool<Item>>(rng: &mut T, num_mutations_to_try: usize) {
    let mut log: Vec<LogEntry> = Vec::new();
    let mut allocations: Allocations = Allocations::new();

    let mut test: U;
    let mut reference: Reference<Item>;
    match generate_random_construction(rng) {
        Construction::New => {
            test = Pool::new();
            reference = Pool::new();
            log.push( LogEntry::Construction( Construction::New ) );

        },

        Construction::WithCapacity{ capacity } => {
            test = Pool::with_capacity(capacity);
            reference = Pool::with_capacity(capacity);
            log.push( LogEntry::Construction( Construction::WithCapacity{ capacity }  ) );
        },
    }

    if let Err(error) = compare_for_equality(&test, &reference, &allocations) {
        panic!("{:?}\n{:?}", error, log);
    }

    /*
        todo: write a get_error() function for every impl of ItemPool and call it
        after each mutation to validate the internal state of test. 
        
        Right now this validates the externally-apparent behavior of each impl, 
        which points to the internal state of test being valid, but doesn't 
        necessarily guarantee it.
    */
    const MAX_NUM_ITEMS_TO_ALLOCATE: usize = 100_000; // don't want to OoM
    let mut generator: MutationGenerator = MutationGenerator::new(rng);
    for _ in 0..num_mutations_to_try {
        match generator.generate(rng) {
            SET => {
                if reference.len() == 0 {
                    continue;
                }
                
                let pair: PairOfIds = allocations.get_random_pair_of_ids(rng).unwrap();
                let id_in_test: usize = pair.id_in_test;
                let id_in_reference: usize = pair.id_in_reference;
                let value: Item = generate_random_item(rng);
                *test.get_mut(id_in_test) = value;
                *reference.get_mut(id_in_reference) = value;

                log.push( 
                    LogEntry::Mutation( 
                        Mutation::Set {
                            id_in_test,
                            id_in_reference, 
                            value
                        } 
                    ) 
                );  
            },

            ALLOCATE => {
                if reference.len() >= MAX_NUM_ITEMS_TO_ALLOCATE {
                    continue;
                }

                let item: Item = generate_random_item(rng);
                let id_in_test: usize = test.allocate(item);
                let id_in_reference: usize = reference.allocate(item);
                allocations.add( 
                    PairOfIds {
                        id_in_test, 
                        id_in_reference
                    }
                );
                
                log.push(
                    LogEntry::Mutation( 
                        Mutation::Allocate { 
                            item
                        } 
                    ) 
                );
            },

            DEALLOCATE => {
                if reference.len() == 0 {
                    continue;
                }

                let pair: PairOfIds = allocations.get_random_pair_of_ids(rng).unwrap();
                let id_in_test: usize = pair.id_in_test;
                let id_in_reference: usize = pair.id_in_reference;
                test.deallocate(id_in_test);
                reference.deallocate(id_in_reference);
                allocations.remove(pair);

                log.push( 
                    LogEntry::Mutation(
                        Mutation::Deallocate {
                            id_in_test, 
                            id_in_reference
                        } 
                    ) 
                );
            },

            _ => unreachable!(),
        }

        if let Err(error) = compare_for_equality(&test, &reference, &allocations) {
            panic!("{:?}\n{:?}", error, log);
        }
    }
}

fn fuzz_many_item_pools<T: Pool<Item>>(num_pools_to_fuzz: usize, max_num_mutations: usize) {
    /*
        Xoshiro256StarStar is a deterministic PRNG that's seeded from the same value every time, 
        which makes these tests consistently reproducible (you get exactly the same sequence of 
        operations every single time this fuzz_many_item_pools() is called)
    */
    const RNG_SEED: u64 = 2049;
    let mut rng: Xoshiro256StarStar = Xoshiro256StarStar::seed_from_u64(RNG_SEED);

    for _ in 0..num_pools_to_fuzz {
        fuzz::<_, T>(&mut rng, max_num_mutations);
    }
}

fn generate_random_item<T: Rng>(rng: &mut T) -> Item {
    return rng.gen_range(Item::MIN..=Item::MAX)
}