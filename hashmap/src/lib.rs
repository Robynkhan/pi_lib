extern crate map;
extern crate fnv;

use std::hash::Hash;

use map::Map;
use fnv::{FnvHashMap};


pub struct HashMap<K, V>(FnvHashMap<K, V>);

impl<K: Hash + Eq, V> Map for HashMap<K, V>{
    type Key = K;
	type Val = V;

    #[inline]
    fn len(&self) -> usize{
        self.0.len()
    }

    #[inline]
    fn contains(&self, key: &Self::Key) -> bool{
        self.0.contains_key(key)
    }

    #[inline]
    fn get(&self, key: &Self::Key) -> Option<&Self::Val> {
        self.0.get(key)
    }

    #[inline]
    fn get_mut(&mut self, key: &Self::Key) -> Option<&mut Self::Val> {
        self.0.get_mut(key)
    }

    #[inline]
    unsafe fn get_unchecked(&self, key: &Self::Key) -> &Self::Val {
        self.0.get(key).unwrap()
    }

    #[inline]
    unsafe fn get_unchecked_mut(&mut self, key: &Self::Key) -> &mut Self::Val {
        self.0.get_mut(key).unwrap()
    }

    #[inline]
    unsafe fn remove_unchecked(&mut self, key: &Self::Key) -> Self::Val {
        self.0.remove(key).unwrap()
    }

    #[inline]
    fn insert(&mut self, key: Self::Key, val: Self::Val) -> Option<Self::Val> {
        self.0.insert(key, val)
    }

    #[inline]
    fn remove(&mut self, key: &Self::Key) -> Option<Self::Val> {
        self.0.remove(key)
    }
}