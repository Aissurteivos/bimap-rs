//! Implementations of `serde::Serialize` and `serde::Deserialize` for
//! `BiHashMap` and `BiBTreeMap`.
//!
//! You do not need to import anything from this module to use this
//! functionality, simply enable the `serde` feature in your dependency
//! manifest.
//!
//! # Examples
//!
//! You can easily serialize and deserialize bimaps with any serde-compatbile
//! `serializer or deserializer.
//!
//! Serializing and deserializing a BiHashMap:
//!
//! ```
//! # use bimap::BiHashMap;
//! // create a new bimap
//! let mut map = BiHashMap::new();
//!
//! // insert some pairs
//! map.insert("A", 1);
//! map.insert("B", 2);
//! map.insert("C", 3);
//!
//! // convert the bimap to json
//! let json = serde_json::to_string(&map).unwrap();
//!
//! // convert the json back into a bimap
//! let map2 = serde_json::from_str(&json).unwrap();
//!
//! // check that the two bimaps are equal
//! assert_eq!(map, map2);
//! ```
//!
//! Serializing and deserializing a BiBTreeMap:
//! ```
//! # use bimap::BiBTreeMap;
//! // create a new bimap
//! let mut map = BiBTreeMap::new();
//!
//! // insert some pairs
//! map.insert(1, 3);
//! map.insert(2, 2);
//! map.insert(3, 1);
//!
//! // convert the bimap to json
//! let json = serde_json::to_string(&map).unwrap();
//!
//! // convert the json back into a bimap
//! let map2 = serde_json::from_str(&json).unwrap();
//!
//! // check that the two bimaps are equal
//! assert_eq!(map, map2);
//! ```
//!
//! Of course, this is only possible for bimaps where the values also implement
//! `Serialize` and `Deserialize` respectively:
//!
//! ```compile_fail
//! # use bimap::BiHashMap;
//! // this type doesn't implement Serialize or Deserialize!
//! #[derive(PartialEq, Eq, Hash)]
//! enum MyEnum { A, B, C }
//!
//! // create a bimap and add some pairs
//! let mut map = BiHashMap::new();
//! map.insert(MyEnum::A, 1);
//! map.insert(MyEnum::B, 2);
//! map.insert(MyEnum::C, 3);
//!
//! // this line will cause the code to fail to compile
//! let json = serde_json::to_string(&map).unwrap();
//! ```
//!
//! Although possible, deserializing a bimap from a serialized form that was
//! not originally a bimap is not recommended or supported. Deserialization of
//! a bimap silently overwrites any pairs where either value is already stored,
//! and this can cause undefined behavior.
//! ```
//! # use std::collections::HashMap;
//! # use bimap::BiHashMap;
//! // construct a regular map
//! let mut map = HashMap::new();
//!
//! // insert some entries
//! // note that both "B" and "C" are associated with the value 2 here
//! map.insert("A", 1);
//! map.insert("B", 2);
//! map.insert("C", 2);
//!
//! // serialize the map
//! let json = serde_json::to_string(&map).unwrap();
//!
//! // deserialize it into a bimap
//! let bimap: BiHashMap<&str, i32> = serde_json::from_str(&json).unwrap();
//!
//! // deserialization succeeds, but the bimap is now in a non-deterministic
//! // state - either ("B", 2) or ("C", 2) will have been overwritten while
//! // deserializing, but this depends on the iteration order of the original
//! // HashMap that was serialized.
//!
//! // we can still demonstrate that certain properties of the bimap are still
//! // in a known state
//! assert_eq!(bimap.len(), 2);
//! assert_eq!(bimap.get_by_left(&"A"), Some(&1));
//! assert!(bimap.get_by_left(&"B") == Some(&2) || bimap.get_by_left(&"C") == Some(&2))
//! ```

use crate::{BiHashMap, BiBTreeMap};
use serde::{Serializer, Serialize, Deserializer, Deserialize};
use serde::de::{Visitor, MapAccess};
use std::hash::Hash;
use std::fmt::{Formatter, Result as FmtResult};
use std::marker::PhantomData;
use std::default::Default;

/// Serializer for `BiHashMap`
impl<L, R> Serialize for BiHashMap<L, R>
where
    L: Serialize + Eq + Hash,
    R: Serialize + Eq + Hash,
{
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_map(self.iter())
    }
}

/// Visitor to construct `BiHashMap` from serialized map entries
struct BiHashMapVisitor<L, R> {
    marker: PhantomData<BiHashMap<L, R>>
}

impl<'de, L, R> Visitor<'de> for BiHashMapVisitor<L, R>
where
    L: Deserialize<'de> + Eq + Hash,
    R: Deserialize<'de> + Eq + Hash,
{
    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "a map")
    }

    type Value = BiHashMap<L, R>;
    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
        let mut map = match entries.size_hint() {
            Some(s) => BiHashMap::with_capacity(s),
            None => BiHashMap::new()
        };
        while let Some((l, r)) = entries.next_entry()? {
            map.insert(l, r);
        }
        Ok(map)
    }
}

/// Deserializer for `BiHashMap`
impl<'de, L, R> Deserialize<'de> for BiHashMap<L, R>
where
    L: Deserialize<'de> + Eq + Hash,
    R: Deserialize<'de> + Eq + Hash,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_map(BiHashMapVisitor { marker: PhantomData::default() })
    }
}

/// Serializer for `BiBTreeMap`
impl<L, R> Serialize for BiBTreeMap<L, R>
where
    L: Serialize + Ord,
    R: Serialize + Ord,
{
    fn serialize<S: Serializer>(&self, ser: S) -> Result<S::Ok, S::Error> {
        ser.collect_map(self.iter())
    }
}

/// Visitor to construct `BiBTreeMap` from serialized map entries
struct BiBTreeMapVisitor<L, R> {
    marker: PhantomData<BiBTreeMap<L, R>>
}

impl<'de, L, R> Visitor<'de> for BiBTreeMapVisitor<L, R>
where
    L: Deserialize<'de> + Ord,
    R: Deserialize<'de> + Ord,
{
    fn expecting(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "a map")
    }

    type Value = BiBTreeMap<L, R>;
    fn visit_map<A: MapAccess<'de>>(self, mut entries: A) -> Result<Self::Value, A::Error> {
        let mut map = BiBTreeMap::new();
        while let Some((l, r)) = entries.next_entry()? {
            map.insert(l, r);
        }
        Ok(map)
    }
}

/// Deserializer for `BiBTreeMap`
impl<'de, L, R> Deserialize<'de> for BiBTreeMap<L, R>
where
    L: Deserialize<'de> + Ord,
    R: Deserialize<'de> + Ord,
{
    fn deserialize<D: Deserializer<'de>>(de: D) -> Result<Self, D::Error> {
        de.deserialize_map(BiBTreeMapVisitor { marker: PhantomData::default() })
    }
}