extern crate fxhash;

use std::hash::{BuildHasherDefault};

use fxhash::FxHasher32;

type FxBuildHasher32 = BuildHasherDefault<FxHasher32>;
pub type FxHashMap32<K, V> = std::collections::HashMap<K, V, FxBuildHasher32>;