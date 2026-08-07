use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use aegis_core::{CoverageBackend, CoverageSnapshot, SessionHandle, TargetConfig, AegisResult, AegisError};

const COVERAGE_MAP_SIZE: usize = 65536;

pub struct CoverageBitmap {
    data: Vec<AtomicU8>,
    virgin_bits: Vec<AtomicU8>,
}

impl CoverageBitmap {
    pub fn new() -> Self {
        Self {
            data: (0..COVERAGE_MAP_SIZE).map(|_| AtomicU8::new(0)).collect(),
            virgin_bits: (0..COVERAGE_MAP_SIZE).map(|_| AtomicU8::new(255)).collect(),
        }
    }
    pub fn record_edge(&self, edge_id: usize) {
        let idx = edge_id % COVERAGE_MAP_SIZE;
        let current = self.data[idx].load(Ordering::Relaxed);
        if current < 255 { self.data[idx].store(current.saturating_add(1), Ordering::Relaxed); }
    }
    pub fn has_new_edges(&self) -> bool {
        for i in 0..COVERAGE_MAP_SIZE {
            if self.data[i].load(Ordering::Relaxed) > 0 && self.virgin_bits[i].load(Ordering::Relaxed) == 255 {
                return true;
            }
        }
        false
    }
    pub fn new_edges(&self) -> Vec<u64> {
        let mut edges = Vec::new();
        for i in 0..COVERAGE_MAP_SIZE {
            if self.data[i].load(Ordering::Relaxed) > 0 && self.virgin_bits[i].load(Ordering::Relaxed) == 255 {
                edges.push(i as u64);
            }
        }
        edges
    }
    pub fn update_virgin_bits(&self) {
        for i in 0..COVERAGE_MAP_SIZE {
            if self.data[i].load(Ordering::Relaxed) > 0 { self.virgin_bits[i].store(0, Ordering::Relaxed); }
        }
    }
    pub fn reset(&self) { for i in 0..COVERAGE_MAP_SIZE { self.data[i].store(0, Ordering::Relaxed); } }
    pub fn edge_count(&self) -> usize {
        (0..COVERAGE_MAP_SIZE).filter(|&i| self.data[i].load(Ordering::Relaxed) > 0).count()
    }
    pub fn total_edges(&self) -> usize {
        (0..COVERAGE_MAP_SIZE).filter(|&i| self.virgin_bits[i].load(Ordering::Relaxed) == 0).count()
    }
    pub fn as_bytes(&self) -> Vec<u8> {
        (0..COVERAGE_MAP_SIZE).map(|i| self.data[i].load(Ordering::Relaxed)).collect()
    }
    pub fn path_hash(&self) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        let mut hasher = DefaultHasher::new();
        for i in 0..COVERAGE_MAP_SIZE {
            let val = self.data[i].load(Ordering::Relaxed);
            if val > 0 { i.hash(&mut hasher); val.hash(&mut hasher); }
        }
        hasher.finish()
    }
}

impl Default for CoverageBitmap { fn default() -> Self { Self::new() } }

static mut GLOBAL_BITMAP: Option<Arc<CoverageBitmap>> = None;

pub fn init_global_bitmap() { unsafe { GLOBAL_BITMAP = Some(Arc::new(CoverageBitmap::new())); } }
pub fn global_bitmap() -> Option<Arc<CoverageBitmap>> { unsafe { GLOBAL_BITMAP.clone() } }

#[no_mangle]
pub extern "C" fn __sanitizer_cov_trace_pc_guard_init(start: *mut u32, stop: *mut u32) {
    if start.is_null() || stop.is_null() { return; }
    unsafe {
        let mut guard = 1u32;
        let mut ptr = start;
        while ptr < stop { *ptr = guard; guard += 1; ptr = ptr.add(1); }
    }
}

#[no_mangle]
pub extern "C" fn __sanitizer_cov_trace_pc_guard(guard: *mut u32) {
    if guard.is_null() { return; }
    unsafe {
        let guard_val = *guard;
        if let Some(bitmap) = global_bitmap() { bitmap.record_edge(guard_val as usize); }
    }
}

pub struct SanCovBackend {
    bitmap: Arc<CoverageBitmap>,
    initialized: bool,
}

impl SanCovBackend {
    pub fn new() -> Self {
        let bitmap = Arc::new(CoverageBitmap::new());
        unsafe { GLOBAL_BITMAP = Some(bitmap.clone()); }
        Self { bitmap, initialized: false }
    }
    pub fn snapshot(&self) -> CoverageSnapshot {
        CoverageSnapshot {
            edge_count: self.bitmap.edge_count(),
            new_edges: self.bitmap.new_edges(),
            total_edges: self.bitmap.total_edges(),
            hit_map: self.bitmap.as_bytes(),
            path_hash: self.bitmap.path_hash(),
        }
    }
    pub fn reset(&self) { self.bitmap.reset(); }
    pub fn update_virgin(&self) { self.bitmap.update_virgin_bits(); }
}

impl Default for SanCovBackend { fn default() -> Self { Self::new() } }

impl CoverageBackend for SanCovBackend {
    fn initialize(&mut self, _target: &TargetConfig) -> AegisResult<()> { self.initialized = true; Ok(()) }
    fn start_session(&mut self) -> AegisResult<SessionHandle> {
        if !self.initialized { return Err(AegisError::CoverageError("Not initialized".into())); }
        self.bitmap.reset(); Ok(SessionHandle::new())
    }
    fn end_session(&mut self, _session: SessionHandle) -> AegisResult<CoverageSnapshot> {
        Ok(self.snapshot())
    }
    fn is_available(&self) -> bool { true }
    fn name(&self) -> &str { "sancov" }
}
