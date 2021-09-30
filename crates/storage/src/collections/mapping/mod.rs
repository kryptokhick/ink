// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A simple mapping to contract storage.

use crate::traits::{
    push_spread_root,
    pull_spread_root,
    clear_spread_root,
    ExtKeyPtr,
    KeyPtr,
    SpreadLayout,
    PackedLayout,
};
use ink_env::hash::{
    Blake2x256,
    HashOutput,
};
use ink_primitives::Key;

use core::marker::PhantomData;

/// A mapping of key-value pairs directly into contract storage.
///
/// If a key does not exist the `Default` value for the `value` will be returned.
pub struct Mapping<K, V> {
    key: Key,
    _phantom_mapping: (PhantomData<K>, PhantomData<V>),
}

impl<K, V> Mapping<K, V>
where
    K: PackedLayout,
    V: PackedLayout + Default,
{
    /// Creates a new empty `Mapping`.
    ///
    /// Not sure how this should be exposed/initialize irl.
    pub fn new(key: Key) -> Self {
        Self {
            key,
            _phantom_mapping: (Default::default(), Default::default()),
        }
    }

    /// Insert the given `value` to the contract storage.
    pub fn insert(&mut self, key: K, value: V) {
        ink_env::set_contract_storage(&self.key(key), &value);
    }

    /// Get the `value` at `key` from the contract storage.
    pub fn get(&self, key: K) -> V {
        ink_env::get_contract_storage(&self.key(key))
            .unwrap_or_default()
            .unwrap_or_default()
    }

    fn key(&self, key: K) -> Key {
        let encodedable_key = (self.key, key);
        let mut output = <Blake2x256 as HashOutput>::Type::default();
        ink_env::hash_encoded::<Blake2x256, _>(&encodedable_key, &mut output);
        output.into()
    }
}

impl<K, V> SpreadLayout for Mapping<K, V> {
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = false;

    #[inline]
    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        let root_key = ExtKeyPtr::next_for::<Self>(ptr);
        pull_spread_root::<Self>(&root_key)
    }

    #[inline]
    fn push_spread(&self, ptr: &mut KeyPtr) {
        let root_key = ExtKeyPtr::next_for::<Self>(ptr);
        push_spread_root::<Self>(self, &root_key);
    }

    #[inline]
    fn clear_spread(&self, ptr: &mut KeyPtr) {
        let root_key = ExtKeyPtr::next_for::<Self>(ptr);
        clear_spread_root::<Self>(self, &root_key);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping = Mapping::new([0u8; 32].into());
            mapping.insert(1, 2);
            assert_eq!(mapping.get(1), 2);

            Ok(())
        }).unwrap()
    }

    #[test]
    fn gets_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            assert_eq!(m1.get(1), u8::default());

            Ok(())
        }).unwrap()
    }
}
