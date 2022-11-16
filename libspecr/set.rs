use crate::specr::*;
use crate::specr::gccow::{GcCow, GcCompat};
use im::hashset::HashSet;
use im::hashmap::HashMap;

use std::any::Any;


pub struct Set<T>(GcCow<HashSet<T>>);

impl<T> Clone for Set<T> {
    fn clone(&self) -> Self { Set(self.0) }
}
impl<T> Copy for Set<T> {}
impl<T> GcCompat for HashSet<T> {
    fn points_to(&self, _m: &mut std::collections::HashSet<usize>) {}
    fn as_any(&self) -> &dyn Any { self }
}

impl<T> GcCompat for Set<T> {
    fn points_to(&self, m: &mut std::collections::HashSet<usize>) {
        self.0.points_to(m);
    }
    fn as_any(&self) -> &dyn Any { self }
}


