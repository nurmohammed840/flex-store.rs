use bin_layout::{Decoder, Encoder};
use std::fmt::Debug;

pub trait Key:
    Encoder + for<'de> Decoder<'de, ()> + PartialOrd + Send + Sync + Unpin + Debug
{
}

pub trait Value:
    Encoder + for<'de> Decoder<'de, ()> + Send + Sync + Unpin + Debug
{
}

macro_rules! impl_for { [$id:ident : $($rty:ty)*] => ($(impl $id for $rty {})*);}

impl Value for () {}

impl<const N: usize> Key for [u8; N] {}
impl<const N: usize> Value for [u8; N] {}

impl_for!(Key: u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64);
impl_for!(Value: u8 u16 u32 u64 u128 i8 i16 i32 i64 i128 f32 f64);
