//! Dynamic type map

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    fmt,
    hash::{BuildHasherDefault, Hasher},
};

/// A TypeId is already a hash, so we don't need to hash it
#[derive(Default)]
struct TypeIdHash(u64);

impl Hasher for TypeIdHash {
    fn write(&mut self, _: &[u8]) {
        unreachable!("TypeId calls write_u64");
    }

    #[inline]
    fn write_u64(&mut self, id: u64) {
        self.0 = id;
    }

    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }
}

type AnyTypeMap = HashMap<TypeId, Box<dyn Any + Send + Sync>, BuildHasherDefault<TypeIdHash>>;

/// This is a HashMap of values, stored based on [`TypeId`]. Every type has a
/// unique `TypeId` generated by the compiler, we are using this id to store in
/// a value in a map of `Box<dyn Any>` based on a type. Then retrieving that
/// value based on its type by downcasting later.
///
/// ```
/// # use dora_core::server::typemap::TypeMap;
/// let mut map = TypeMap::new();
/// map.insert(10_usize);
/// assert_eq!(map.get::<usize>().unwrap(), &10_usize);
/// assert_eq!(map.remove::<usize>().unwrap(), 10_usize);
/// ```
///
/// [`TypeId`]: std::any::TypeId
#[derive(Default)]
pub struct TypeMap {
    map: Option<Box<AnyTypeMap>>,
}

impl TypeMap {
    /// Make a new `TypeMap`, does zero allocation
    #[inline]
    pub fn new() -> TypeMap {
        TypeMap { map: None }
    }

    /// Insert a type into the map. If the type already exists, it will be
    /// returned.
    ///
    /// ```
    /// # use dora_core::server::typemap::TypeMap;
    /// let mut map = TypeMap::new();
    /// assert!(map.insert(10_usize).is_none());
    /// assert!(map.insert(10_u8).is_none());
    /// assert_eq!(map.insert(15_usize), Some(10_usize));
    /// ```
    pub fn insert<T: Send + Sync + 'static>(&mut self, val: T) -> Option<T> {
        self.map
            .get_or_insert_with(|| Box::new(HashMap::default()))
            .insert(TypeId::of::<T>(), Box::new(val))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|x| *x)
            })
    }

    /// Get a reference to a type previously inserted
    ///
    /// ```
    /// # use dora_core::server::typemap::TypeMap;
    /// let mut map = TypeMap::new();
    /// assert!(map.get::<i32>().is_none());
    /// map.insert(5i32);
    ///
    /// assert_eq!(map.get::<i32>(), Some(&5i32));
    /// ```
    pub fn get<T: Send + Sync + 'static>(&self) -> Option<&T> {
        self.map
            .as_ref()
            .and_then(|map| map.get(&TypeId::of::<T>()))
            .and_then(|boxed| (**boxed).downcast_ref::<T>())
    }

    /// Get a mutable reference to a type previously inserted
    ///
    /// ```
    /// # use dora_core::server::typemap::TypeMap;
    /// let mut map = TypeMap::new();
    /// map.insert(String::from("Hello"));
    /// map.get_mut::<String>().unwrap().push_str(" World");
    ///
    /// assert_eq!(map.get::<String>().unwrap(), "Hello World");
    /// ```
    pub fn get_mut<T: Send + Sync + 'static>(&mut self) -> Option<&mut T> {
        self.map
            .as_mut()
            .and_then(|map| map.get_mut(&TypeId::of::<T>()))
            .and_then(|boxed| (**boxed).downcast_mut())
    }

    /// Remove a type
    ///
    /// ```
    /// # use dora_core::server::typemap::TypeMap;
    /// let mut map = TypeMap::new();
    /// map.insert(10_usize);
    /// assert_eq!(map.remove::<usize>(), Some(10_usize));
    /// assert!(map.get::<usize>().is_none());
    /// ```
    pub fn remove<T: Send + Sync + 'static>(&mut self) -> Option<T> {
        self.map
            .as_mut()
            .and_then(|map| map.remove(&TypeId::of::<T>()))
            .and_then(|boxed| {
                (boxed as Box<dyn Any + 'static>)
                    .downcast()
                    .ok()
                    .map(|x| *x)
            })
    }

    /// Clear the `TypeMap` of all inserted values.
    ///
    /// ```
    /// # use dora_core::server::typemap::TypeMap;
    /// let mut map = TypeMap::new();
    /// map.insert(10_usize);
    /// map.clear();
    ///
    /// assert!(map.get::<usize>().is_none());
    /// ```
    pub fn clear(&mut self) {
        if let Some(ref mut map) = self.map {
            map.clear();
        }
    }
}

impl fmt::Debug for TypeMap {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("TypeMap").finish()
    }
}
