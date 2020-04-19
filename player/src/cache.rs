use game_sdk::Action;

pub const HASH_SIZE: usize = 64; //IN MB
pub struct Cache {
    pub entries: usize,
    pub buckets: usize,
    pub cache: Vec<CacheBucket>,
}
impl Cache {
    pub fn with_size(size: usize) -> Cache {
        let buckets = 1024 * 1024 * size / 64;
        let entries = buckets * 3;
        let cache = vec![CacheBucket([CacheEntry::invalid(); 3]); buckets];
        Cache {
            entries,
            buckets,
            cache,
        }
    }
    pub fn fill_status(&self) -> usize {
        if self.entries < 1000 {
            return 1000;
        }
        let mut counted_entries = 0;
        let mut full = 0;
        let mut index = 0;
        while counted_entries < 500 {
            let bucket = self.cache.get(index).unwrap();
            index += 1;
            full += bucket.fill_status();
            counted_entries += 3;
        }
        let mut index = self.buckets - 1;
        while counted_entries < 1000 {
            let bucket = self.cache.get(index).unwrap();
            full += bucket.fill_status();
            counted_entries += 3;
            index -= 1;
        }
        (full as f64 / counted_entries as f64 * 1000.0) as usize
    }

    pub fn lookup(&self, hash: u64) -> Option<CacheEntry> {
        self.cache[hash as usize % self.buckets].probe(hash)
    }

    pub fn insert(&mut self, hash: u64, ce: CacheEntry, rootplies: u8) {
        self.cache
            .get_mut(hash as usize % self.buckets)
            .unwrap()
            .should_replace(ce, hash, rootplies);
    }
}
#[repr(align(64))]
#[derive(Clone)]
pub struct CacheBucket([CacheEntry; 3]);
impl CacheBucket {
    pub fn fill_status(&self) -> usize {
        let mut res = 0;
        for i in 0..3 {
            if !self.0[i].is_invalid() {
                res += 1;
            }
        }
        res
    }

    pub fn probe(&self, hash: u64) -> Option<CacheEntry> {
        if self.0[0].valid_hash(hash) {
            return Some(self.0[0]);
        } else if self.0[1].valid_hash(hash) {
            Some(self.0[1])
        } else if self.0[2].valid_hash(hash) {
            Some(self.0[2])
        } else {
            None
        }
    }

    pub fn should_replace(&mut self, ce: CacheEntry, hash: u64, rootplies: u8) {
        //Slot 0 is highest depth
        //Slot 1 is random replace
        //Slot 2 is always
        if self.0[0].is_invalid()
            || self.0[0].plies < rootplies
            || self.0[0].valid_hash(hash)
            || ce.depth >= self.0[0].depth
        {
            self.0[0] = ce;
            return;
        } else if self.0[1].is_invalid()
            || self.0[1].plies < rootplies
            || self.0[1].valid_hash(hash)
            || ce.depth + 1 >= self.0[1].depth
        {
            self.0[1] = ce;
        } else if self.0[2].is_invalid()
            || self.0[2].plies < rootplies
            || self.0[2].valid_hash(hash)
            || ce.depth + 3 >= self.0[2].depth
        {
            self.0[2] = ce;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct CacheEntry {
    pub upper_hash: u32, //4
    pub lower_hash: u32, //8
    pub action: Action,  //12
    pub score: i16,      //14
    pub depth: u8,       //15
    pub alpha: bool,     //16
    pub beta: bool,      //17
    pub plies: u8,       //18
}
impl CacheEntry {
    pub fn valid_hash(&self, hash: u64) -> bool {
        (self.upper_hash as u64) == (hash >> 32) && (self.lower_hash) as u64 == (hash & 0xFFFFFFFF)
    }
    pub fn is_invalid(&self) -> bool {
        self.depth == std::u8::MAX
    }
    pub fn invalid() -> CacheEntry {
        CacheEntry {
            upper_hash: 0,
            lower_hash: 0,
            action: Action::SkipMove,
            score: 0,
            depth: std::u8::MAX,
            alpha: false,
            beta: false,
            plies: 0,
        }
    }
}

pub struct EvalCache {
    pub entries: usize,
    pub buckets: usize,
    pub cache: Vec<EvalCacheBucket>,
}
impl EvalCache {
    pub fn with_size(size: usize) -> EvalCache {
        let buckets = 1024 * 1024 * size / 64;
        let entries = buckets * 5;
        let cache = vec![EvalCacheBucket([EvalCacheEntry::invalid(); 5]); buckets];
        EvalCache {
            entries,
            buckets,
            cache,
        }
    }
    pub fn fill_status(&self) -> usize {
        if self.entries < 1000 {
            return 1000;
        }
        let mut counted_entries = 0;
        let mut full = 0;
        let mut index = 0;
        while counted_entries < 500 {
            let bucket = self.cache.get(index).unwrap();
            index += 1;
            full += bucket.fill_status();
            counted_entries += 5;
        }
        let mut index = self.buckets - 1;
        while counted_entries < 1000 {
            let bucket = self.cache.get(index).unwrap();
            full += bucket.fill_status();
            counted_entries += 5;
            index -= 1;
        }
        (full as f64 / counted_entries as f64 * 1000.0) as usize
    }

    pub fn lookup(&self, hash: u64) -> Option<EvalCacheEntry> {
        self.cache[hash as usize % self.buckets].probe(hash)
    }

    pub fn insert(&mut self, hash: u64, ce: EvalCacheEntry) {
        self.cache
            .get_mut(hash as usize % self.buckets)
            .unwrap()
            .should_replace(ce);
    }
}

#[repr(align(64))]
#[derive(Clone)]
pub struct EvalCacheBucket([EvalCacheEntry; 5]);
impl EvalCacheBucket {
    pub fn fill_status(&self) -> usize {
        let mut res = 0;
        for i in 0..3 {
            if !self.0[i].is_invalid() {
                res += 1;
            }
        }
        res
    }

    pub fn probe(&self, hash: u64) -> Option<EvalCacheEntry> {
        if self.0[0].valid_hash(hash) {
            Some(self.0[0])
        } else if self.0[1].valid_hash(hash) {
            Some(self.0[1])
        } else if self.0[2].valid_hash(hash) {
            Some(self.0[2])
        } else if self.0[3].valid_hash(hash) {
            Some(self.0[3])
        } else if self.0[4].valid_hash(hash) {
            Some(self.0[4])
        } else {
            None
        }
    }

    pub fn should_replace(&mut self, ce: EvalCacheEntry) {
        //Slot 0 is highest depth
        //Slot 1 is random replace
        //Slot 2 is always
        if self.0[0].is_invalid() {
            self.0[0] = ce;
        } else if self.0[1].is_invalid() {
            self.0[1] = ce;
        } else if self.0[2].is_invalid() {
            self.0[2] = ce;
        } else if self.0[3].is_invalid() {
            self.0[3] = ce;
        } else if self.0[4].is_invalid() {
            self.0[4] = ce;
        } else {
            self.0[4] = self.0[3];
            self.0[3] = self.0[2];
            self.0[2] = self.0[1];
            self.0[1] = self.0[0];
            self.0[0] = ce;
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct EvalCacheEntry {
    pub upper_hash: u32, //4
    pub lower_hash: u32, //8
    pub score: i16,      //12
}
impl EvalCacheEntry {
    pub fn valid_hash(&self, hash: u64) -> bool {
        (self.upper_hash as u64) == (hash >> 32) && (self.lower_hash) as u64 == (hash & 0xFFFFFFFF)
    }
    pub fn is_invalid(&self) -> bool {
        self.score == std::i16::MIN
    }
    pub fn invalid() -> Self {
        EvalCacheEntry {
            upper_hash: 0,
            lower_hash: 0,
            score: std::i16::MIN,
        }
    }
}
