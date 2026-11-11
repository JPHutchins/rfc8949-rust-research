#![no_std]
extern crate alloc;
use alloc::boxed::Box;
use core::mem::MaybeUninit;
pub struct Frame(u64);
pub trait Nested { type Stack; const MAX_DEPTH: usize; }
#[repr(C)] pub struct Level<C> { _f: MaybeUninit<Frame>, _c: C }
pub struct List { pub next: Option<Box<List>> }
impl<T: Nested> Nested for Option<Box<T>> { type Stack = T::Stack; const MAX_DEPTH: usize = T::MAX_DEPTH; }
impl Nested for List { type Stack = Level<<Option<Box<List>> as Nested>::Stack>; const MAX_DEPTH: usize = 1 + <Option<Box<List>> as Nested>::MAX_DEPTH; }
pub fn size() -> usize { core::mem::size_of::<<List as Nested>::Stack>() }
