use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;
use std::hash::Hash;
use std::mem;
use std::borrow::Borrow;

const INTIAL_BUCKETS: usize = 1;

pub struct HashMap <K, V> {
    buckets: Vec<Vec<(K, V)>>,
    items: usize
}



impl <K,V> HashMap<K, V> {
    pub fn new() -> Self {
        HashMap {
            buckets: Vec::new(),
            items: 0
        }
    }
}

pub struct OccupiedEntry<'a, K: 'a, V: 'a>{
    element: &'a mut (K, V)
}
pub struct VacantEntry<'a, K: 'a, V: 'a>{
    key: K,
    bucket: &'a mut Vec<(K, V)>
}

impl <'a, K: 'a, V: 'a> VacantEntry<'a, K, V> {
    pub fn insert(self, value: V) -> &'a mut V {
        self.bucket.push((self.key, value));
        &mut self.bucket.last_mut().unwrap().1
    }
}

pub enum Entry<'a, K: 'a, V: 'a> {
    Occupied(OccupiedEntry<'a, K, V>),
    Vacant(VacantEntry<'a, K, V>)
}

impl<'a, K: 'a, V: 'a> Entry<'a, K, V> {
    pub fn or_insert(self, value: V) -> &'a mut V {
        match self {
            Entry::Occupied(e) => {
                &mut e.element.1
            },
            Entry::Vacant(e) => {
                e.insert(value)
            }
        }
    }

    pub fn or_insert_with<F>(self, maker: F) -> &'a mut V 
    where F: FnOnce() -> V
    {
        match self {
            Entry::Occupied(e) => {
                &mut e.element.1
            },
            Entry::Vacant(e) => {
                e.insert(maker())
            }
        }
    }

    pub fn or_insert_default<F>(self) -> &'a mut V 
    where V: Default
    {
        match self {
            Entry::Occupied(e) => {
                &mut e.element.1
            },
            Entry::Vacant(e) => {
                e.insert(V::default())
            }
        }
    }
}

impl <K,V> HashMap<K, V> 
where K: Hash + Eq
{

    pub fn bucket<Q>(&mut self, key: &Q) -> usize 
    where 
    K: Borrow<Q>,
    Q: Hash + Eq + ?Sized
    {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        (hasher.finish() % self.buckets.len() as u64) as usize
    }

    // pub fn entry<'a>(&'a mut self, key: K) -> Entry<'a, K, V> {
    //     let bucket = self.bucket(&key);
    //     // let bucket = &mut self.buckets[bucket];

    //     if let Some(entry) = self.buckets[bucket]
    //         .iter_mut()
    //         .find(|&(ref ekey, ref value)| ekey == &key) {
    //             return Entry::Occupied(OccupiedEntry {element: unsafe {&mut *(entry as &mut _)}})
    //         } 
    //         Entry::Vacant(
    //             VacantEntry{key, bucket: &mut self.buckets[bucket]}
    //         )
    // }

    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        if self.items > 3 * self.buckets.len() / 4 || self.buckets.is_empty() {
            self.resize()
        }
        
        let bucket = self.bucket(&key);
        let bucket = &mut self.buckets[bucket];
        self.items += 1;

        for &mut (ref ekey, ref mut evalue) in bucket.iter_mut() {
            if ekey == &key {
                return Some(mem::replace(evalue, value))
            }
        }

        bucket.push((key, value));
        None
    }

    pub fn get<Q>(&mut self, key: &Q) -> Option<&V> 
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let bucket = self.bucket(key);
        self.buckets[bucket]
            .iter()
            .find(|&(ref ekey, _)| ekey.borrow() == key)
            .map(|&(_, ref v)| v)
    }

    // pub fn entry(&mut self, key: K, value: V) -> Entry<K, V> {

    // }

    pub fn contains_key<Q>(&mut self, key: &Q) -> bool 
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        self.get(key).is_some()
    }

    pub fn remove<Q>(&mut self, key: &Q) -> Option<V> 
    where 
        K: Borrow<Q>,
        Q: Hash + Eq + ?Sized
    {
        let bucket = self.bucket(key);
        let bucket = &mut self.buckets[bucket];
        let i = bucket
                .iter()
                .position(|&(ref ekey, _)| ekey.borrow() == key)?;
        self.items -= 1;
        Some(bucket.swap_remove(i).1)
    }

    pub fn len(&self) -> usize {
        self.items
    }

    pub fn is_empty(&self) -> bool {
        self.items == 0
    }

    pub fn resize(&mut self) {
        let target_size  = match self.buckets.len() {
            0 => INTIAL_BUCKETS,
            n => 2 *n
        };

        let mut new_buckets = Vec::with_capacity(target_size);
        new_buckets.extend((0..target_size).map(|_| Vec::new()));

        for (key, value) in self.buckets.iter_mut().flat_map(|bucket| bucket.drain(..)) {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            let bucket = (hasher.finish() % new_buckets.len() as u64) as usize;
            new_buckets[bucket].push((key, value));
        }

        mem::replace(&mut self.buckets, new_buckets);
    }
}

pub struct Iter<'a, K: 'a, V: 'a> {
    map: &'a HashMap<K, V>,
    bucket: usize,
    at: usize
}

impl <'a, K, V> Iterator for Iter<'a, K, V> {
    type Item = (&'a K, &'a V);
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.map.buckets.get(self.bucket) {
                Some(bucket) => {
                    match bucket.get(self.at) {
                        Some(&(ref key, ref eval)) => {
                            self.at += 1;
                            break Some((key, eval));
                        },
                        None => { 
                            self.at = 0;
                            self.bucket += 1;
                            continue;
                        }
                    }
                },
                None => break None
            }
        }
    }
}

impl <'a, K, V>IntoIterator for &'a HashMap<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            map: self,
            bucket: 0,
            at: 0
        }
    }
}

pub fn test<T: std::fmt::Debug + Hash + Eq + Clone, V: std::fmt::Debug>(k: T, v: V) {
    let mut buc: Vec<Vec<(T, V)>> = Vec::new();
    let mut new = Vec::new();
    new.push((k.clone(), v));
    buc.push(new);
    buc.push(Vec::new());
    buc.push(Vec::new());
    buc.push(Vec::new());
    println!("{:?}", &buc);
    // let num = 0;
    // let num = &mut buc[num];
    // println!("{:?}", num);
    let mut hasher = DefaultHasher::new();
    51.hash(&mut hasher);
    println!("{}", (hasher.finish() % buc.len() as u64) as usize);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    // #[ignore]
    fn insert() {
        let mut map = HashMap::new();
        assert_eq!(map.len(), 0);
        assert_eq!(map.is_empty(), true);
        map.insert("normal", 42);
        assert_eq!(map.contains_key(&"normal"), true);
        assert_eq!(map.get(&"normal"), Some(&42));
        assert_eq!(map.remove(&"normal"), Some(42));
        assert_eq!(map.remove(&"normal"), None);
    }

    #[test]
    #[ignore]
    fn mytest() {
        test(34, 23);
        assert_eq!(true, false)
    }

    #[test]
    fn iter() {
        let mut map = HashMap::new();
        map.insert("foo", 42);
        map.insert("bar", 44);
        map.insert("sha", 41);
        map.insert("you", 40);

        for (&k, &v) in &map {
            match k {
                "foo" => assert_eq!(v, 42),
                "bar" => assert_eq!(v, 44),
                "sha" => assert_eq!(v, 41),
                "you" => assert_eq!(v, 40),
                _ => unreachable!()
            }
        }
    }
}