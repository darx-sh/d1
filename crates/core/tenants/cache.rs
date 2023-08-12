use arrayvec::ArrayVec;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

#[derive(Debug)]
struct Entry<K, V> {
  key: K,
  prev: Option<usize>,
  next: Option<usize>,
  value: V,
}

pub(crate) struct LruCache<K, V, const CAP: usize> {
  locations: HashMap<K, usize>,
  head: Option<usize>,
  tail: Option<usize>,
  values: ArrayVec<Entry<K, V>, CAP>,
}

impl<K, V, const CAP: usize> LruCache<K, V, CAP>
where
  K: Hash + Eq + Clone,
{
  pub fn new() -> Self {
    Self {
      locations: HashMap::new(),
      values: ArrayVec::new(),
      head: None,
      tail: None,
    }
  }

  pub fn put(&mut self, key: K, value: V) -> Option<V> {
    let mut ret = None;

    if let Some(idx) = self.locations.get(&key) {
      let old_val = mem::replace(&mut self.values[*idx].value, value);
      ret = Some(old_val);
      self.move_to_head(*idx);
    } else {
      if self.values.is_empty() {
        self.head = Some(0);
        self.tail = Some(0);
        self.values.push(Entry {
          key: key.clone(),
          prev: None,
          next: None,
          value,
        });
      } else if self.values.is_full() {
        let origin_tail = self.tail.unwrap();
        self.locations.remove(&self.values[origin_tail].key);
        self.values[origin_tail].key = key.clone();
        self.values[origin_tail].value = value;
        self.move_to_head(origin_tail);
      } else {
        let origin_head = self.head.unwrap();
        self.values.push(Entry {
          key: key.clone(),
          prev: None,
          next: Some(origin_head),
          value,
        });
        self.move_to_head(self.values.len() - 1);
      }

      self.locations.insert(key, self.head.unwrap());
    }
    ret
  }

  #[allow(dead_code)]
  pub fn get(&mut self, key: &K) -> Option<&V> {
    if let Some(index) = self.locations.get(key) {
      let index = *index;

      if self.len() != 1 {
        self.move_to_head(index);
      }

      Some(&self.values[index].value)
    } else {
      None
    }
  }

  pub fn get_mut(&mut self, key: &K) -> Option<&mut V> {
    if let Some(index) = self.locations.get(key) {
      let index = *index;

      if self.len() != 1 {
        self.move_to_head(index);
      }

      Some(&mut self.values[index].value)
    } else {
      None
    }
  }

  pub fn len(&self) -> usize {
    self.values.len()
  }

  fn move_to_head(&mut self, index: usize) {
    let e = &mut self.values[index];
    let prev = e.prev;
    let next = e.next;

    if let Some(prev) = prev {
      self.values[prev].next = next;
    } else {
      self.head = next;
    }

    if let Some(next) = next {
      self.values[next].prev = prev;
    } else {
      self.tail = prev;
    }

    let origin_head = self.head;
    let e = &mut self.values[index];
    e.prev = None;
    e.next = origin_head;

    self.head = Some(index);

    if let Some(origin_head) = origin_head {
      self.values[origin_head].prev = self.head;
    }
  }
}

#[cfg(test)]
mod tests {
  use super::LruCache;

  fn assert_opt_eq<V: PartialEq>(opt: Option<&V>, v: V) {
    assert!(opt.is_some());
    assert!(opt.unwrap() == &v);
  }

  #[test]
  fn test_put_and_get() {
    let mut cache: LruCache<_, _, 2> = LruCache::new();
    cache.put(1, 10);
    cache.put(2, 20);
    assert_opt_eq(cache.get(&1), 10);
    assert_opt_eq(cache.get(&2), 20);
    assert_eq!(cache.len(), 2);
  }

  #[test]
  fn test_put_update() {
    let mut cache: LruCache<String, Vec<u8>, 1> = LruCache::new();
    cache.put("1".to_string(), vec![10, 10]);
    cache.put("1".to_string(), vec![10, 19]);
    assert_opt_eq(cache.get(&"1".to_string()), vec![10, 19]);
    assert_eq!(cache.len(), 1);
  }

  #[test]
  fn test_expire_lru() {
    let mut cache: LruCache<String, String, 2> = LruCache::new();
    cache.put("foo1".to_string(), "bar1".to_string());
    cache.put("foo2".to_string(), "bar2".to_string());
    cache.put("foo3".to_string(), "bar3".to_string());
    assert!(cache.get(&"foo1".to_string()).is_none());
    cache.put("foo2".to_string(), "bar2update".to_string());
    cache.put("foo4".to_string(), "bar4".to_string());
    assert!(cache.get(&"foo3".to_string()).is_none());
  }

  #[test]
  fn example() {
    let mut cache: LruCache<_, _, 2> = LruCache::new();

    cache.put("cow", 3);
    cache.put("pig", 2);

    assert_eq!(*cache.get(&"cow").unwrap(), 3);
    assert_eq!(*cache.get(&"pig").unwrap(), 2);
    assert!(cache.get(&"dog").is_none());

    assert_eq!(cache.put("pig", 4), Some(2));
    assert_eq!(cache.put("dog", 5), None);

    assert_eq!(*cache.get(&"dog").unwrap(), 5);
    assert_eq!(*cache.get(&"pig").unwrap(), 4);
    assert!(cache.get(&"cow").is_none());

    {
      let v = cache.get_mut(&"pig").unwrap();
      *v = 6;
    }

    assert_eq!(*cache.get(&"pig").unwrap(), 6);
  }
}
