use rustc_data_structures::fx::FxHasher;
use std::hash::BuildHasherDefault;

pub type FxIndexSet<V> = indexmap::IndexSet<V, BuildHasherDefault<FxHasher>>;
pub type FxIndexMap<K, V> = indexmap::IndexMap<K, V, BuildHasherDefault<FxHasher>>;

pub use rustc_data_structures::fx::FxHashMap;

pub use rustc_data_structures::smallvec::SmallVec;
pub use rustc_data_structures::thin_vec::ThinVec;
