#![no_std]
pub trait Depth { const MAX_DEPTH: usize; }
pub struct Frame(u32);
pub struct Shallow;
impl Depth for Shallow { const MAX_DEPTH: usize = 3; }
pub fn decode<T: Depth, const N: usize>(stack: &mut [Frame; N]) -> usize {
    const { assert!(N >= T::MAX_DEPTH, "stack too small for T") };
    stack.len()
}
pub fn bad(s: &mut [Frame; 2]) -> usize { decode::<Shallow, 2>(s) }
