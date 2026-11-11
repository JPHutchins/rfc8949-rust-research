#![no_std]
use core::mem::{size_of, ManuallyDrop, MaybeUninit};

#[derive(Clone, Copy)]
pub struct Frame { _descriptor: u32, _remaining: u32 }

pub trait Nested {
    type Stack;
    const MAX_DEPTH: usize;
}

#[repr(C)]
pub struct Level<Children> { _frame: MaybeUninit<Frame>, _children: Children }

#[repr(C)]
pub union Widest2<A, B> { _a: ManuallyDrop<A>, _b: ManuallyDrop<B> }

const fn max(a: usize, b: usize) -> usize { if a > b { a } else { b } }

impl Nested for u32 { type Stack = (); const MAX_DEPTH: usize = 0; }

pub struct Leaf { pub a: u32, pub b: u32 }
impl Nested for Leaf {
    type Stack = Level<()>;
    const MAX_DEPTH: usize = 1 + 0;
}

pub struct Mid { pub leaf: Leaf, pub n: u32 }
impl Nested for Mid {
    type Stack = Level<<Leaf as Nested>::Stack>;
    const MAX_DEPTH: usize = 1 + <Leaf as Nested>::MAX_DEPTH;
}

pub struct Top { pub mid: Mid, pub leaf: Leaf }
impl Nested for Top {
    type Stack = Level<Widest2<<Mid as Nested>::Stack, <Leaf as Nested>::Stack>>;
    const MAX_DEPTH: usize = 1 + max(<Mid as Nested>::MAX_DEPTH, <Leaf as Nested>::MAX_DEPTH);
}

pub struct Envelope<T> { pub hdr: Leaf, pub body: T }
impl<T: Nested> Nested for Envelope<T> {
    type Stack = Level<Widest2<<Leaf as Nested>::Stack, T::Stack>>;
    const MAX_DEPTH: usize = 1 + max(<Leaf as Nested>::MAX_DEPTH, T::MAX_DEPTH);
}

pub const fn stack_matches<T: Nested>() -> bool {
    size_of::<T::Stack>() == T::MAX_DEPTH * size_of::<Frame>()
}

const _: () = assert!(stack_matches::<Leaf>());
const _: () = assert!(stack_matches::<Mid>());
const _: () = assert!(stack_matches::<Top>());
const _: () = assert!(stack_matches::<Envelope<Top>>());
const _: () = assert!(stack_matches::<Envelope<u32>>());
const _: () = assert!(<Top as Nested>::MAX_DEPTH == 3);
const _: () = assert!(<Envelope<Top> as Nested>::MAX_DEPTH == 4);

pub struct Decoder<T: Nested> { pub stack: MaybeUninit<T::Stack> }

pub fn frames<T: Nested>(decoder: &mut Decoder<T>) -> &mut [MaybeUninit<Frame>] {
    const { assert!(stack_matches::<T>()) };
    unsafe { core::slice::from_raw_parts_mut(decoder.stack.as_mut_ptr().cast(), T::MAX_DEPTH) }
}

pub const SIZES: [usize; 4] = [
    size_of::<Decoder<Leaf>>(),
    size_of::<Decoder<Top>>(),
    size_of::<Decoder<Envelope<Top>>>(),
    size_of::<Decoder<u32>>(),
];
