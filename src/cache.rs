//! # Cache

// Structures.
pub struct Cache([i32; 0x10000]);
// Implementations.
impl Cache {
    pub fn new() -> Self {
        Self([0; 0x10000])
    }
    pub fn inner_mut(&mut self) -> &mut [i32; 0x10000] {
        &mut self.0
    }
}
