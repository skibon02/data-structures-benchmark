use std::alloc::{GlobalAlloc, Layout, System};
use std::array::from_fn;
use std::collections::{BTreeMap, HashMap};
use std::{fs, mem};
use std::hash::Hash;
use std::hint::black_box;
use std::io::Write;
use std::sync::atomic::{compiler_fence, AtomicUsize, Ordering};
use indexmap::IndexMap;

#[global_allocator]
static GLOBAL:  CountingAllocator = CountingAllocator;
static ALLOCATED: AtomicUsize = AtomicUsize::new(0);

pub struct CountingAllocator;
unsafe impl GlobalAlloc for CountingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size();
        ALLOCATED.fetch_add(size, Ordering::Relaxed);
        System.alloc(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size();
        ALLOCATED.fetch_sub(size, Ordering::Relaxed);
        System.dealloc(ptr, layout)
    }

    unsafe fn realloc(
        &self,
        ptr: *mut u8,
        old: Layout,
        new_size: usize,
    ) -> *mut u8 {
        let old_size = old.size();
        ALLOCATED.fetch_sub(old_size, Ordering::Relaxed);
        ALLOCATED.fetch_add(new_size, Ordering::Relaxed);
        System.realloc(ptr, old, new_size)
    }
}

pub struct Tracker(AtomicUsize);
impl Tracker {
    pub fn new() -> Self {
        Tracker(
            ALLOCATED.load(Ordering::SeqCst).into()
        )
    }
    pub fn allocated(&self) -> usize {
        ALLOCATED.load(Ordering::SeqCst) - self.0.load(Ordering::SeqCst)
    }


    pub fn print(&self, tag: &str) -> usize {
        println!("Heap allocation {}: {}", tag, self.allocated());
        self.allocated()
    }
}

pub trait DataStruct {
    fn new_with_size(size: usize) -> Self;
}

impl<T: Default> DataStruct for Vec<T> {
    fn new_with_size(size: usize) -> Self {
        Vec::with_capacity(size)
    }
}
fn usize_to_bytes<const N: usize>(i: usize) -> [u8; N] {
    let mut k = [0u8; N];
    if N >= 4 {
        k[..4].copy_from_slice(&i.to_le_bytes()[..4]);
    } else if N == 3 {
        k[..3].copy_from_slice(&i.to_le_bytes()[..3])
    }
    else if N == 2 {
        k[..2].copy_from_slice(&i.to_le_bytes()[..2]);
    }
    else {
        k[0] =  i as u8;
    }
    k
}
impl<const N: usize, V: Default> DataStruct for BTreeMap<[u8; N], V> {
    fn new_with_size(size: usize) -> Self {
        let mut r = BTreeMap::new();
        for i in 0..size {
            r.insert(usize_to_bytes(i), V::default());
        }
        black_box(&r);
        r
    }
}
impl<const N: usize, V: Default> DataStruct for IndexMap<[u8; N], V> {
    fn new_with_size(size: usize) -> Self {
        let mut r = IndexMap::new();
        for i in 0..size {
            r.insert(usize_to_bytes(i), V::default());
        }
        black_box(&r);
        r
    }
}
impl<const N: usize, V: Default> DataStruct for HashMap<[u8; N], V> {
    fn new_with_size(size: usize) -> Self {
        let mut r = HashMap::new();
        for i in 0..size {
            r.insert(usize_to_bytes(i), V::default());
        }
        black_box(&r);
        r
    }
}

pub struct TestResults {
    res: BTreeMap<(usize, usize, usize), usize>,
    name: String,
}
impl TestResults {
    pub fn new(name: String) -> Self {
        Self {
            res: BTreeMap::new(),
            name,
        }
    }
    
    pub fn run_tests<K: Eq + Ord + Hash, V, S: DataStruct>(&mut self, tracker: &Tracker, sizes: impl Iterator<Item=usize> + Clone) {
        let k_sz = size_of::<K>();
        let v_sz = size_of::<V>();
        println!("Running for {}, K={}, V={}", self.name , k_sz, v_sz);
        for len in sizes {
            let ovh_size = test_structure::<S>(tracker, len, k_sz + v_sz);
            self.res.insert((len, k_sz, v_sz), ovh_size);
        }
    }
    
    pub fn save_csv(&mut self) {
        let filename =  format!("{}.csv", self.name);
        let mut file = fs::File::create(filename).unwrap();
        file.write_all(b"length, k_sz, v_sz, ovh_size\n").unwrap();
        for ((length, k_sz, v_sz), ovh_size) in mem::take(&mut self.res) {
            file.write_all(format!("{},{},{},{}\n",length, k_sz, v_sz, ovh_size).as_bytes()).unwrap()
        }
    }
}

fn main() {
    let s2 : Vec<usize> = DataStruct::new_with_size(2);
    println!("{:?}", s2);
    drop(s2);
    let tracker  = Tracker::new();
    tracker.print("START");

    let mut btreemap_res = TestResults::new("BTreeMap".to_string());
    let mut hashmap_res = TestResults::new("HashMap".to_string());
    let mut indexmap_res = TestResults::new("IndexMap".to_string());
    
    macro_rules! run_tests {
        ($sizes:expr, $tracker:expr, $k_sz:literal, $v_sz:literal) => {
            {
                type K = [u8; $k_sz];
                type V = [u8; $v_sz];
                hashmap_res.run_tests::<K, V, HashMap<K,V>>(&$tracker, $sizes.clone());
                indexmap_res.run_tests::<K, V, IndexMap<K,V>>(&$tracker, $sizes.clone());
                btreemap_res.run_tests::<K, V, BTreeMap<K,V>>(&$tracker, $sizes.clone());
            }
        };
    }

    let sizes_1 = (0..256).step_by(5);
    let sizes_2 = (0..256).step_by(5).chain((500..65_000).step_by(2_000));
    run_tests!{sizes_1, tracker, 1, 1};
    run_tests!{sizes_1, tracker, 1, 2};
    run_tests!{sizes_1, tracker, 1, 4};
    run_tests!{sizes_1, tracker, 1, 8};
    run_tests!(sizes_2, tracker, 2, 1);
    run_tests!(sizes_2, tracker, 2, 2);
    run_tests!(sizes_2, tracker, 2, 4);
    run_tests!(sizes_2, tracker, 2, 8);
    run_tests!(sizes_2, tracker, 4, 1);
    run_tests!(sizes_2, tracker, 4, 2);
    run_tests!(sizes_2, tracker, 4, 4);
    run_tests!(sizes_2, tracker, 4, 8);
    

    btreemap_res.save_csv();
    indexmap_res.save_csv();
    hashmap_res.save_csv();
    
    tracker.print("END");
}

fn test_structure<S: DataStruct>(tracker: &Tracker, len: usize, el_size: usize) -> usize {
    let before = tracker.allocated();
    compiler_fence(Ordering::AcqRel);
    let s: S = DataStruct::new_with_size(len);
    compiler_fence(Ordering::AcqRel);
    let bytes = tracker.allocated() - before;
    let ovh = bytes - (len * el_size);
    // println!("{} - {} : {}b ({}%)", len, len * el_size, ovh, (bytes as f32 / (len * el_size) as f32 - 1.0) * 100.0);
    println!("{} - {}", len * el_size, bytes);
    ovh
}