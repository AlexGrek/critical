use std::{
    collections::HashSet,
    sync::{Arc},
};

use gitops_lib::store::{
    StorageError,
    qstorage::{KvStorage, StorageResult},
};

pub struct IndexView {
    storage: Arc<dyn KvStorage>,
    store: &'static str,
}

impl IndexView {
    /// Creates a new `IndexView` bound to a specific store name.
    ///
    /// The caller is responsible for ensuring the store is initialized.
    pub fn new(storage: Arc<dyn KvStorage>, store:&'static str ) -> Self {
        Self { storage, store }
    }

    /// Helper to get items, taking a locked storage guard to prevent re-locking.
    fn _get_or_empty(&self, key: &str) -> StorageResult<Vec<String>> {
        match self.storage.get(&self.store, key) {
            Ok(items) => Ok(items),
            Err(StorageError::ItemNotFound { .. }) => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    /// Appends a string to the list if it's not already present.
    #[must_use]
    pub fn append_unique(&self, key: &str, item: &str) -> StorageResult<()> {
        let mut items = self._get_or_empty(key)?;
        if !items.iter().any(|i| i == item) {
            items.push(item.to_string());
            self.storage.set(self.store, key, items)?;
        }
        Ok(())
    }

    /// Appends multiple strings from an iterator, skipping any that are already present.
    #[must_use]
    pub fn append_unique_list<'i, I>(&self, key: &str, new_items: I) -> StorageResult<()>
    where
        I: IntoIterator<Item = &'i str>,
    {
        let items = self._get_or_empty(key)?;
        let mut existing_set: HashSet<String> = items.into_iter().collect();
        let original_len = existing_set.len();

        existing_set.extend(new_items.into_iter().map(String::from));

        if existing_set.len() > original_len {
            // Convert the set back to a Vec for storage.
            let updated_items: Vec<String> = existing_set.into_iter().collect();
            self.storage.set(self.store, key, updated_items)?;
        }

        Ok(())
    }

    /// Removes a specific string from the list. Does nothing if the item is not found.
    #[must_use]
    pub fn remove(&self, key: &str, item_to_remove: &str) -> StorageResult<()> {
        let mut items = self._get_or_empty(key)?;
        let original_len = items.len();
        items.retain(|i| i != item_to_remove);

        if items.len() < original_len {
            self.storage.set(self.store, key, items)?;
        }
        Ok(())
    }

    /// Removes multiple strings from the list.
    #[must_use]
    pub fn remove_list<'i, I>(&self, key: &str, items_to_remove: I) -> StorageResult<()>
    where
        I: IntoIterator<Item = &'i str>,
    {
        let mut items = self._get_or_empty(key)?;
        let original_len = items.len();
        let to_remove_set: HashSet<_> = items_to_remove.into_iter().collect();

        if to_remove_set.is_empty() {
            return Ok(());
        }

        items.retain(|i| !to_remove_set.contains(i.as_str()));

        if items.len() < original_len {
            self.storage.set(self.store, key, items)?;
        }
        Ok(())
    }

    /// Removes an item, returning an `ItemNotFound` error if it wasn't present.
    #[must_use]
    pub fn remove_or_fail(&self, key: &str, item_to_remove: &str) -> StorageResult<()> {
        let mut items = self.storage.get(self.store, key)?;
        let original_len = items.len();

        items.retain(|i| i != item_to_remove);

        if items.len() == original_len {
            return Err(StorageError::ItemNotFound {
                key: item_to_remove.to_string(),
                kind: "item in list".to_string(),
            });
        }

        self.storage.set(self.store, key, items)
    }

    /// Checks if the list contains a specific item.
    #[must_use]
    pub fn contains(&self, key: &str, item: &str) -> StorageResult<bool> {
        let items = self._get_or_empty(key)?;
        Ok(items.iter().any(|i| i == item))
    }

    /// Checks if the list contains any of the specified items.
    #[must_use]
    pub fn contains_any<'i, I>(&self, key: &str, items_to_check: I) -> StorageResult<bool>
    where
        I: IntoIterator<Item = &'i str>,
    {
        let items = self._get_or_empty(key)?;
        if items.is_empty() {
            return Ok(false);
        }
        let existing_set: HashSet<_> = items.iter().map(String::as_str).collect();
        Ok(items_to_check.into_iter().any(|i| existing_set.contains(i)))
    }

    /// Checks if the list contains all of the specified items.
    #[must_use]
    pub fn contains_all<'i, I>(&self, key: &str, items_to_check: I) -> StorageResult<bool>
    where
        I: IntoIterator<Item = &'i str>,
    {
        let items = self._get_or_empty(key)?;
        if items.is_empty() {
            // If we need to check for any items, it can't contain them.
            // This is a bit ambiguous, so we check if the iterator is empty.
            return Ok(items_to_check.into_iter().next().is_none());
        }
        let existing_set: HashSet<_> = items.iter().map(String::as_str).collect();
        Ok(items_to_check.into_iter().all(|i| existing_set.contains(i)))
    }

    /// Appends an item without checking for uniqueness.
    #[must_use]
    pub fn append_copy(&mut self, key: &str, item: &str) -> StorageResult<()> {
        let mut items = self._get_or_empty(key)?;
        items.push(item.to_string());
        self.storage.set(self.store, key, items)
    }

    /// Appends a list of items without checking for uniqueness.
    #[must_use]
    pub fn append_copy_list<'i, I>(&self, key: &str, new_items: I) -> StorageResult<()>
    where
        I: IntoIterator<Item = &'i str>,
    {
        let mut items = self._get_or_empty(key)?;
        items.extend(new_items.into_iter().map(String::from));
        self.storage.set(self.store, key, items)
    }

    /// Removes all items for a key, setting the value to an empty list.
    #[must_use]
    pub fn clear(&mut self, key: &str) -> StorageResult<()> {
        self.storage.set(self.store, key, Vec::new())
    }

    /// Removes all items for a key and returns them. Returns an empty Vec if key was not present.
    #[must_use]
    pub fn drain(&self, key: &str) -> StorageResult<Vec<String>> {
        let items = self._get_or_empty(key)?;
        if !items.is_empty() {
            self.storage.set(self.store, key, Vec::new())?;
        }
        Ok(items)
    }

    /// Returns the first `len` items from the list.
    #[must_use]
    pub fn take_first(&self, key: &str, len: usize) -> StorageResult<Vec<String>> {
        let items = self._get_or_empty(key)?;
        Ok(items.into_iter().take(len).collect())
    }

    /// Returns the number of items in the list, or 0 if the key does not exist.
    #[must_use]
    pub fn len(&self, key: &str) -> StorageResult<usize> {
        self._get_or_empty(key).map(|items| items.len())
    }

    /// Retrieves all items for a key. Returns an empty Vec if the key is not found.
    #[must_use]
    pub fn get_all(&self, key: &str) -> StorageResult<Vec<String>> {
        self._get_or_empty(key)
    }

    /// Retrieves all items and returns them as a `HashSet`.
    #[must_use]
    pub fn get_all_as_set(&self, key: &str) -> StorageResult<HashSet<String>> {
        self._get_or_empty(key)
            .map(|items| items.into_iter().collect())
    }

    /// Checks if a key exists. Returns `false` on any error, including `ItemNotFound`.
    pub fn key_exists(&self, key: &str) -> bool {
        self.storage.get(self.store, key).is_ok()
    }

    /// Replaces the data for a key with new data, returning the old data.
    #[must_use]
    pub fn swap(&self, key: &str, new_data: Vec<String>) -> StorageResult<Vec<String>> {
        let old_data = self._get_or_empty(key)?;
        self.storage.set(self.store, key, new_data)?;
        Ok(old_data)
    }

    /// Calls `append_unique` for an item across multiple keys.
    #[must_use]
    pub fn append_unique_to_all<'k, I>(&self, keys: I, item: &str) -> StorageResult<()>
    where
        I: IntoIterator<Item = &'k str>,
    {
        for key in keys {
            self.append_unique(key, item)?;
        }
        Ok(())
    }

    /// Calls `remove` for an item across multiple keys.
    #[must_use]
    pub fn remove_from_all<'k, I>(&self, keys: I, item: &str) -> StorageResult<()>
    where
        I: IntoIterator<Item = &'k str>,
    {
        for key in keys {
            self.remove(key, item)?;
        }
        Ok(())
    }

    /// Calls `append_unique_list` for multiple items across multiple keys.
    #[must_use]
    pub fn append_unique_to_all_list<'k, 'i, K, I>(
        &self,
        keys: K,
        items: I,
    ) -> StorageResult<()>
    where
        K: IntoIterator<Item = &'k str>,
        I: IntoIterator<Item = &'i str> + Clone,
    {
        for key in keys {
            self.append_unique_list(key, items.clone())?;
        }
        Ok(())
    }

    /// Calls `remove_list` for multiple items across multiple keys.
    #[must_use]
    pub fn remove_from_all_list<'k, 'i, K, I>(&self, keys: K, items: I) -> StorageResult<()>
    where
        K: IntoIterator<Item = &'k str>,
        I: IntoIterator<Item = &'i str> + Clone,
    {
        for key in keys {
            self.remove_list(key, items.clone())?;
        }
        Ok(())
    }
}
