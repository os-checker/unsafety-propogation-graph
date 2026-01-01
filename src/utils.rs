use rustc_data_structures::fx::FxHasher;
use std::hash::BuildHasherDefault;

pub type FxIndexSet<V> = indexmap::IndexSet<V, BuildHasherDefault<FxHasher>>;
pub type FxIndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;
