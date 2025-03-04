#[cfg(test)]
mod tests;

mod system;
pub use mem::MaybeUninit;
use num::{bigint::ParseBigIntError, Integer, Num, One, Signed};
pub use once_cell::unsync::Lazy;
use std::{
    any::Any,
    borrow::Borrow,
    cell::{RefCell, UnsafeCell},
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::{Debug, Display, Formatter},
    hash::{Hash, Hasher},
    mem,
    ops::{Add, Deref, Div, Mul, Neg, Rem, Sub},
    rc::{Rc, Weak},
};

pub use system::*;

pub use itertools;
pub use num::bigint::BigInt;
pub use num::rational::BigRational;
pub use num::FromPrimitive;
pub use num::NumCast;
pub use num::ToPrimitive;
pub use num::Zero;
pub use std::convert::Into;

// An atomic box is just a RefCell in Rust
pub type SizeT = usize;

pub trait DafnyType: Clone + DafnyPrint + 'static {}

impl<T> DafnyType for T where T: Clone + DafnyPrint + 'static {}
pub trait DafnyTypeEq: DafnyType + Hash + Eq {}

impl<T> DafnyTypeEq for T where T: DafnyType + Hash + Eq {}

// Dafny's type (0) compiles to NontrivialDefault to prevent subset types from being considered as Default if their witness is nonzero
pub trait NontrivialDefault {
    fn nontrivial_default() -> Self;
}

pub mod dafny_runtime_conversions {
    use crate::DafnyType;
    use crate::DafnyTypeEq;
    pub type DafnyInt = crate::DafnyInt;
    pub type DafnySequence<T> = crate::Sequence<T>;
    pub type DafnyMap<K, V> = crate::Map<K, V>;
    pub type DafnySet<T> = crate::Set<T>;
    pub type DafnyMultiset<T> = crate::Multiset<T>;
    pub type DafnyBool = bool;
    pub type DafnyChar = crate::DafnyChar;
    pub type DafnyCharUTF16 = crate::DafnyCharUTF16;
    pub type DafnyClass<T> = *mut T;
    pub type DafnyArray<T> = *mut [T];
    pub type DafnyArray2<T> = *mut crate::Array2<T>;
    pub type DafnyArray3<T> = *mut crate::Array3<T>;

    use num::BigInt;
    use num::ToPrimitive;

    use std::collections::HashMap;
    use std::collections::HashSet;
    use std::hash::Hash;
    use std::rc::Rc;

    // Conversion to and from Dafny classes. All these methods take ownership of the class.
    pub unsafe fn dafny_class_to_struct<T: Clone>(ptr: DafnyClass<T>) -> T {
        *dafny_class_to_boxed_struct(ptr)
    }
    pub unsafe fn dafny_class_to_boxed_struct<T: Clone>(ptr: DafnyClass<T>) -> Box<T> {
        Box::from_raw(ptr)
    }
    pub fn struct_to_dafny_class<T>(t: T) -> DafnyClass<T> {
        boxed_struct_to_dafny_class(Box::new(t))
    }
    pub fn boxed_struct_to_dafny_class<T>(t: Box<T>) -> DafnyClass<T> {
        Box::into_raw(t)
    }

    // Conversions to and from Dafny arrays. They all take ownership
    pub unsafe fn dafny_array_to_vec<T: Clone>(ptr: DafnyArray<T>) -> Vec<T> {
        ptr.as_ref().unwrap().to_vec()
    }
    pub fn vec_to_dafny_array<T: Clone>(array: Vec<T>) -> DafnyArray<T> {
        Box::into_raw(array.into_boxed_slice())
    }
    pub unsafe fn dafny_array2_to_vec<T: Clone>(ptr: DafnyArray2<T>) -> Vec<Vec<T>> {
        Box::from_raw(ptr).to_vec()
    }

    pub fn dafny_int_to_bigint(i: &DafnyInt) -> BigInt {
        i.data.as_ref().clone()
    }
    pub fn bigint_to_dafny_int(i: &BigInt) -> DafnyInt {
        DafnyInt {
            data: Rc::new(i.clone()),
        }
    }

    pub fn dafny_sequence_to_vec<T, X>(s: &DafnySequence<T>, elem_converter: fn(&T) -> X) -> Vec<X>
    where
        T: DafnyType,
    {
        let mut array: Vec<T> = Vec::with_capacity(s.cardinality_usize());
        DafnySequence::<T>::append_recursive(&mut array, s);
        array.iter().map(|x| elem_converter(x)).collect()
    }

    // Used for external conversions
    pub fn vec_to_dafny_sequence<T, X>(
        array: &Vec<X>,
        elem_converter: fn(&X) -> T,
    ) -> DafnySequence<T>
    where
        T: DafnyType,
    {
        let mut result: Vec<T> = Vec::with_capacity(array.len());
        for elem in array.iter() {
            result.push(elem_converter(elem));
        }
        DafnySequence::<T>::from_array_owned(result)
    }

    pub fn dafny_map_to_hashmap<K, V, K2, V2>(
        m: &DafnyMap<K, V>,
        converter_k: fn(&K) -> K2,
        converter_v: fn(&V) -> V2,
    ) -> HashMap<K2, V2>
    where
        K: DafnyTypeEq,
        V: DafnyTypeEq,
        K2: Eq + Hash,
        V2: Clone,
    {
        m.to_hashmap_owned(converter_k, converter_v)
    }

    pub fn hashmap_to_dafny_map<K2, V2, K, V>(
        map: &HashMap<K2, V2>,
        converter_k: fn(&K2) -> K,
        converter_v: fn(&V2) -> V,
    ) -> DafnyMap<K, V>
    where
        K: DafnyTypeEq,
        V: DafnyTypeEq,
        K2: Eq + Hash,
        V2: Clone,
    {
        DafnyMap::<K, V>::from_hashmap(map, converter_k, converter_v)
    }

    // --unicode-chars:true
    pub mod unicode_chars_true {
        use crate::Sequence;

        type DafnyChar = crate::DafnyChar;
        type DafnyString = Sequence<DafnyChar>;

        pub fn string_to_dafny_string(s: &str) -> DafnyString {
            Sequence::from_array_owned(s.chars().map(|v| crate::DafnyChar(v)).collect())
        }
        pub fn dafny_string_to_string(s: &DafnyString) -> String {
            let characters = s.to_array();
            characters.iter().map(|v| v.0).collect::<String>()
        }
    }

    // --unicode-chars:false
    pub mod unicode_chars_false {
        use crate::Sequence;

        type DafnyCharUTF16 = crate::DafnyCharUTF16;
        type DafnyString = Sequence<DafnyCharUTF16>;

        pub fn string_to_dafny_string(s: &str) -> DafnyString {
            Sequence::from_array_owned(s.encode_utf16().map(|v| crate::DafnyCharUTF16(v)).collect())
        }
        pub fn dafny_string_to_string(s: &DafnyString) -> String {
            let characters = s
                .to_array()
                .as_ref()
                .iter()
                .map(|v| v.0)
                .collect::<Vec<_>>();
            String::from_utf16_lossy(&characters)
        }
    }

    pub fn set_to_dafny_set<U, T: DafnyTypeEq>(
        set: &HashSet<U>,
        converter: fn(&U) -> T,
    ) -> DafnySet<T> {
        DafnySet::from_iterator(set.iter().map(converter))
    }
    pub fn dafny_set_to_set<T, U>(set: &DafnySet<T>, converter: fn(&T) -> U) -> HashSet<U>
    where
        T: DafnyTypeEq,
        U: Clone + Eq + Hash,
    {
        let mut result: HashSet<U> = HashSet::new();
        for s in set.data.iter() {
            result.insert(converter(s));
        }
        result
    }

    pub fn dafny_multiset_to_owned_vec<T, U>(
        ms: &DafnyMultiset<T>,
        converter: fn(&T) -> U,
    ) -> Vec<U>
    where
        T: DafnyTypeEq,
        U: Clone + Eq,
    {
        let mut result: Vec<U> = Vec::new();
        for s in ms.data.iter() {
            // Push T as many times as its size
            for _ in 0..s.1.data.to_usize().unwrap() {
                result.push(converter(&s.0));
            }
        }
        result
    }

    pub fn vec_to_dafny_multiset<T, U>(vec: &Vec<U>, converter: fn(&U) -> T) -> DafnyMultiset<T>
    where
        T: DafnyTypeEq,
        U: Clone + Eq + Hash,
    {
        DafnyMultiset::from_iterator(vec.into_iter().map(|u: &U| converter(u)))
    }
}

// **************
// Dafny integers
// **************

// Zero-cost abstraction over a Rc<BigInt>
#[derive(Clone)]
pub struct DafnyInt {
    data: Rc<BigInt>,
}

impl DafnyInt {
    pub fn new(data: Rc<BigInt>) -> DafnyInt {
        DafnyInt { data }
    }
    pub fn as_usize(&self) -> usize {
        self.to_usize().unwrap()
    }
}

impl AsRef<BigInt> for DafnyInt {
    fn as_ref(&self) -> &BigInt {
        &self.data
    }
}

// truncate_u(x, u64)
// = <DafnyInt as ToPrimitive>::to_u128(&x).unwrap() as u64;
#[macro_export]
macro_rules! truncate {
    ($x:expr, $t:ty) => {
        <$crate::DafnyInt as $crate::Into<$t>>::into($x)
    };
}

impl Into<u8> for DafnyInt {
    fn into(self) -> u8 {
        self.data.to_u8().unwrap()
    }
}
impl Into<u16> for DafnyInt {
    fn into(self) -> u16 {
        self.data.to_u16().unwrap()
    }
}
impl Into<u32> for DafnyInt {
    fn into(self) -> u32 {
        self.data.to_u32().unwrap()
    }
}
impl Into<u64> for DafnyInt {
    fn into(self) -> u64 {
        self.data.to_u64().unwrap()
    }
}
impl Into<u128> for DafnyInt {
    fn into(self) -> u128 {
        self.data.to_u128().unwrap()
    }
}
impl Into<i8> for DafnyInt {
    fn into(self) -> i8 {
        self.data.to_i8().unwrap()
    }
}
impl Into<i16> for DafnyInt {
    fn into(self) -> i16 {
        self.data.to_i16().unwrap()
    }
}
impl Into<i32> for DafnyInt {
    fn into(self) -> i32 {
        self.data.to_i32().unwrap()
    }
}
impl Into<i64> for DafnyInt {
    fn into(self) -> i64 {
        self.data.to_i64().unwrap()
    }
}
impl Into<i128> for DafnyInt {
    fn into(self) -> i128 {
        self.data.to_i128().unwrap()
    }
}

impl ToPrimitive for DafnyInt {
    fn to_i64(&self) -> Option<i64> {
        self.data.to_i64()
    }

    fn to_u64(&self) -> Option<u64> {
        self.data.to_u64()
    }

    // Override of functions
    fn to_u128(&self) -> Option<u128> {
        self.data.to_u128()
    }

    fn to_i128(&self) -> Option<i128> {
        self.data.to_i128()
    }
}

impl Default for DafnyInt {
    fn default() -> Self {
        DafnyInt::new(Rc::new(BigInt::zero()))
    }
}

impl NontrivialDefault for DafnyInt {
    fn nontrivial_default() -> Self {
        Self::default()
    }
}

impl PartialEq<DafnyInt> for DafnyInt {
    fn eq(&self, other: &DafnyInt) -> bool {
        self.data.eq(&other.data)
    }
}
impl Eq for DafnyInt {}
impl Hash for DafnyInt {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}

impl DafnyPrint for DafnyInt {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl ::std::fmt::Debug for DafnyInt {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.data)
    }
}

impl Add<DafnyInt> for DafnyInt {
    type Output = DafnyInt;

    fn add(self, rhs: DafnyInt) -> Self::Output {
        DafnyInt {
            data: Rc::new(self.data.as_ref() + rhs.data.as_ref()),
        }
    }
}

impl Mul<DafnyInt> for DafnyInt {
    type Output = DafnyInt;

    fn mul(self, rhs: DafnyInt) -> Self::Output {
        DafnyInt {
            data: Rc::new(self.data.as_ref() * rhs.data.as_ref()),
        }
    }
}

impl Div<DafnyInt> for DafnyInt {
    type Output = DafnyInt;

    fn div(self, rhs: DafnyInt) -> Self::Output {
        DafnyInt {
            data: Rc::new(self.data.as_ref() / rhs.data.as_ref()),
        }
    }
}

impl Sub<DafnyInt> for DafnyInt {
    type Output = DafnyInt;

    fn sub(self, rhs: DafnyInt) -> Self::Output {
        DafnyInt {
            data: Rc::new(self.data.as_ref() - rhs.data.as_ref()),
        }
    }
}
impl Rem<DafnyInt> for DafnyInt {
    type Output = DafnyInt;

    fn rem(self, rhs: DafnyInt) -> Self::Output {
        DafnyInt {
            data: Rc::new(self.data.as_ref() % rhs.data.as_ref()),
        }
    }
}
impl Neg for DafnyInt {
    type Output = DafnyInt;

    #[inline]
    fn neg(self) -> Self::Output {
        DafnyInt {
            data: Rc::new(-self.data.as_ref()),
        }
    }
}
impl Zero for DafnyInt {
    #[inline]
    fn zero() -> Self {
        DafnyInt {
            data: Rc::new(BigInt::zero()),
        }
    }
    #[inline]
    fn is_zero(&self) -> bool {
        self.data.is_zero()
    }
}
impl One for DafnyInt {
    #[inline]
    fn one() -> Self {
        DafnyInt {
            data: Rc::new(BigInt::one()),
        }
    }
}
impl Num for DafnyInt {
    type FromStrRadixErr = ParseBigIntError;

    #[inline]
    fn from_str_radix(s: &str, radix: u32) -> Result<Self, Self::FromStrRadixErr> {
        Ok(DafnyInt {
            data: Rc::new(BigInt::from_str_radix(s, radix)?),
        })
    }
}
impl Ord for DafnyInt {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.data.cmp(&other.data)
    }
}
impl Signed for DafnyInt {
    #[inline]
    fn abs(&self) -> Self {
        DafnyInt {
            data: Rc::new(self.data.as_ref().abs()),
        }
    }

    #[inline]
    fn abs_sub(&self, other: &Self) -> Self {
        DafnyInt {
            data: Rc::new(self.data.as_ref().abs_sub(other.data.as_ref())),
        }
    }

    #[inline]
    fn signum(&self) -> Self {
        DafnyInt {
            data: Rc::new(self.data.as_ref().signum()),
        }
    }

    #[inline]
    fn is_positive(&self) -> bool {
        self.data.as_ref().is_positive()
    }

    #[inline]
    fn is_negative(&self) -> bool {
        self.data.as_ref().is_negative()
    }
}

// Comparison
impl PartialOrd<DafnyInt> for DafnyInt {
    #[inline]
    fn partial_cmp(&self, other: &DafnyInt) -> Option<Ordering> {
        self.data.partial_cmp(&other.data)
    }
}

impl DafnyInt {
    #[inline]
    pub fn parse_bytes(number: &[u8], radix: u32) -> DafnyInt {
        DafnyInt {
            data: ::std::rc::Rc::new(BigInt::parse_bytes(number, radix).unwrap()),
        }
    }
    pub fn from_usize(usize: usize) -> DafnyInt {
        DafnyInt {
            data: Rc::new(BigInt::from(usize)),
        }
    }
    pub fn from_i32(i: i32) -> DafnyInt {
        DafnyInt {
            data: Rc::new(BigInt::from(i)),
        }
    }
}

macro_rules! impl_dafnyint_from {
    () => {};
    ($type:ident) => {
        impl From<$type> for DafnyInt {
            fn from(n: $type) -> Self {
                DafnyInt {
                    data: Rc::new(n.into()),
                }
            }
        }
    };
}

impl_dafnyint_from! { u8 }
impl_dafnyint_from! { u16 }
impl_dafnyint_from! { u32 }
impl_dafnyint_from! { u64 }
impl_dafnyint_from! { u128 }
impl_dafnyint_from! { i8 }
impl_dafnyint_from! { i16 }
impl_dafnyint_from! { i32 }
impl_dafnyint_from! { i64 }
impl_dafnyint_from! { i128 }
impl_dafnyint_from! { usize }

impl<'a> From<&'a [u8]> for DafnyInt {
    fn from(number: &[u8]) -> Self {
        DafnyInt::parse_bytes(number, 10)
    }
}

// Now the same but for &[u8, N] for any kind of such references
impl<'a, const N: usize> From<&'a [u8; N]> for DafnyInt {
    fn from(number: &[u8; N]) -> Self {
        DafnyInt::parse_bytes(number, 10)
    }
}

// **************
// Immutable sequences
// **************

impl<T: DafnyTypeEq> Eq for Sequence<T> {}

impl<T: DafnyType> Add<&Sequence<T>> for &Sequence<T> {
    type Output = Sequence<T>;

    fn add(self, rhs: &Sequence<T>) -> Self::Output {
        Sequence::new_concat_sequence(self, rhs)
    }
}

impl<T: DafnyTypeEq> Hash for Sequence<T> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.cardinality_usize().hash(state);
        let array = self.to_array();
        // Iterate over the elements
        for elt in array.iter() {
            elt.hash(state);
        }
    }
}

// Clone can be derived automatically
#[derive(Clone)]
pub enum Sequence<T>
where
    T: DafnyType,
{
    ArraySequence {
        // Values could be a native array because we will know statically that all
        // accesses are in bounds when using this data structure
        values: Rc<Vec<T>>,
    },
    ConcatSequence {
        left: Rc<UnsafeCell<Sequence<T>>>,
        right: Rc<UnsafeCell<Sequence<T>>>,
        length: SizeT,
        boxed: Rc<RefCell<Option<Rc<Vec<T>>>>>,
    },
}

impl<T> Sequence<T>
where
    T: DafnyType,
{
    pub fn from_array(values: &Vec<T>) -> Sequence<T> {
        Sequence::ArraySequence {
            values: Rc::new(values.clone()),
        }
    }
    pub fn from_array_slice(values: &Vec<T>, start: &DafnyInt, end: &DafnyInt) -> Sequence<T> {
        Sequence::ArraySequence {
            values: Rc::new(values[start.to_usize().unwrap()..end.to_usize().unwrap()].to_vec()),
        }
    }
    pub fn from_array_take(values: &Vec<T>, n: &DafnyInt) -> Sequence<T> {
        Sequence::ArraySequence {
            values: Rc::new(values[..n.to_usize().unwrap()].to_vec()),
        }
    }
    pub fn from_array_drop(values: &Vec<T>, n: &DafnyInt) -> Sequence<T> {
        Sequence::ArraySequence {
            values: Rc::new(values[n.to_usize().unwrap()..].to_vec()),
        }
    }
    pub fn from_array_owned(values: Vec<T>) -> Sequence<T> {
        Sequence::ArraySequence {
            values: Rc::new(values),
        }
    }
    pub fn new_concat_sequence(left: &Sequence<T>, right: &Sequence<T>) -> Sequence<T> {
        Sequence::ConcatSequence {
            left: Rc::new(UnsafeCell::new(left.clone())),
            right: Rc::new(UnsafeCell::new(right.clone())),
            length: left.cardinality_usize() + right.cardinality_usize(),
            boxed: Rc::new(RefCell::new(None)),
        }
    }
    pub fn to_array(&self) -> Rc<Vec<T>> {
        // Let's convert the if then else below to a proper match statement
        match self {
            Sequence::ArraySequence { values, .. } =>
            // The length of the elements
            {
                Rc::clone(values)
            }
            Sequence::ConcatSequence {
                length,
                boxed,
                left,
                right,
            } => {
                let into_boxed = boxed.as_ref().clone();
                let into_boxed_borrowed = into_boxed.borrow();
                let borrowed: Option<&Rc<Vec<T>>> = into_boxed_borrowed.as_ref();
                if let Some(cache) = borrowed.as_ref() {
                    return Rc::clone(cache);
                }
                // Let's create an array of size length and fill it up recursively
                // We don't materialize nested arrays because most of the time they are forgotten
                let mut array: Vec<T> = Vec::with_capacity(*length);
                Sequence::<T>::append_recursive(&mut array, self);
                let result = Rc::new(array);
                let mut cache = boxed.borrow_mut();
                let mutable_left: *mut Sequence<T> = left.get();
                let mutable_right: *mut Sequence<T> = right.get();
                // safety: Once the array is computed, left and right won't ever be read again.
                unsafe { *mutable_left = seq!() };
                unsafe { *mutable_right = seq!() };
                *cache = Some(result.clone());
                result
            }
        }
    }

    pub fn append_recursive(array: &mut Vec<T>, this: &Sequence<T>) {
        match this {
            Sequence::ArraySequence { values, .. } =>
            // The length of the elements
            {
                for value in values.iter() {
                    array.push(value.clone());
                }
            }
            Sequence::ConcatSequence {
                boxed, left, right, ..
            } =>
            // Let's create an array of size length and fill it up recursively
            {
                let into_boxed = boxed.as_ref().clone();
                let into_boxed_borrowed = into_boxed.borrow();
                let borrowed: Option<&Rc<Vec<T>>> = into_boxed_borrowed.as_ref();
                if let Some(values) = borrowed.as_ref() {
                    for value in values.iter() {
                        array.push(value.clone());
                    }
                    return;
                }
                // safety: When a concat is initialized, the left and right are well defined
                Sequence::<T>::append_recursive(array, unsafe { &mut *left.get() });
                Sequence::<T>::append_recursive(array, unsafe { &mut *right.get() });
            }
        }
    }
    /// Returns the cardinality of this [`Sequence<T>`].
    // The cardinality returns the length of the sequence
    pub fn cardinality_usize(&self) -> SizeT {
        match self {
            Sequence::ArraySequence { values, .. } =>
            // The length of the elements
            {
                values.len()
            }
            Sequence::ConcatSequence { length, .. } => *length,
        }
    }
    pub fn cardinality(&self) -> DafnyInt {
        DafnyInt::from_usize(self.cardinality_usize())
    }
    pub fn get_usize(&self, index: SizeT) -> T {
        let array = self.to_array();
        array[index].clone()
    }

    pub fn slice(&self, start: &DafnyInt, end: &DafnyInt) -> Sequence<T> {
        let start_index = start.data.as_ref().to_usize().unwrap();
        let end_index = end.data.as_ref().to_usize().unwrap();
        let new_data = Sequence::from_array_owned(self.to_array()[start_index..end_index].to_vec());
        new_data
    }
    pub fn take(&self, end: &DafnyInt) -> Sequence<T> {
        let end_index = end.data.as_ref().to_usize().unwrap();
        let new_data = Sequence::from_array_owned(self.to_array()[..end_index].to_vec());
        new_data
    }
    pub fn drop(&self, start: &DafnyInt) -> Sequence<T> {
        let start_index = start.data.as_ref().to_usize().unwrap();
        let new_data = Sequence::from_array_owned(self.to_array()[start_index..].to_vec());
        new_data
    }

    pub fn update_index(&self, index: &DafnyInt, value: &T) -> Self {
        let mut result = self.to_array().as_ref().clone();
        result[index.data.to_usize().unwrap()] = value.clone();
        Sequence::from_array_owned(result)
    }

    pub fn concat(&self, other: &Sequence<T>) -> Sequence<T> {
        Sequence::new_concat_sequence(self, other)
    }

    pub fn get(&self, index: &DafnyInt) -> T {
        self.get_usize(index.data.to_usize().unwrap())
    }
    pub fn iter(&self) -> SequenceIter<T> {
        SequenceIter {
            array: self.to_array(),
            index: 0,
        }
    }
}

pub struct SequenceIter<T: Clone> {
    array: Rc<Vec<T>>,
    index: SizeT,
}
impl<T: Clone> Iterator for SequenceIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.array.len() {
            let result = self.array[self.index].clone();
            self.index += 1;
            Some(result)
        } else {
            None
        }
    }
}

impl<T: DafnyType> Default for Sequence<T> {
    fn default() -> Self {
        Sequence::from_array_owned(vec![])
    }
}
impl<T: DafnyType> NontrivialDefault for Sequence<T> {
    fn nontrivial_default() -> Self {
        Self::default()
    }
}

impl<T: DafnyTypeEq> Sequence<T> {
    pub fn as_dafny_multiset(&self) -> Multiset<T> {
        Multiset::from_array(&self.to_array())
    }
}

// Makes it possible to write iterator.collect::<Sequence<T>> and obtain a sequence
impl<T: DafnyType> FromIterator<T> for Sequence<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Sequence::from_array_owned(iter.into_iter().collect())
    }
}

impl<T: DafnyTypeEq> Sequence<T> {
    pub fn contains(&self, value: &T) -> bool {
        self.to_array().contains(value)
    }
}
impl<T> PartialEq<Sequence<T>> for Sequence<T>
where
    T: DafnyTypeEq,
{
    fn eq(&self, other: &Sequence<T>) -> bool {
        // Iterate through both elements and verify that they are equal
        let values: Rc<Vec<T>> = self.to_array();
        if other.cardinality_usize() != values.len() {
            return false;
        }
        let mut i: usize = 0;
        for value in values.iter() {
            if value != &other.get_usize(i) {
                return false;
            }
            i += 1;
        }
        true
    }
}

impl<T: DafnyTypeEq> PartialOrd for Sequence<T> {
    fn partial_cmp(&self, other: &Sequence<T>) -> Option<Ordering> {
        // Comparison is only prefix-based
        match self.cardinality_usize().cmp(&other.cardinality_usize()) {
            Ordering::Equal => {
                if self == other {
                    Some(Ordering::Equal)
                } else {
                    None
                }
            }
            Ordering::Less => {
                for i in 0..self.cardinality_usize() {
                    if self.get_usize(i) != other.get_usize(i) {
                        return None;
                    }
                }
                Some(Ordering::Less)
            }
            Ordering::Greater => {
                for i in 0..other.cardinality_usize() {
                    if self.get_usize(i) != other.get_usize(i) {
                        return None;
                    }
                }
                Some(Ordering::Greater)
            }
        }
    }
}

impl<V: DafnyType> DafnyPrint for Sequence<V> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        if !V::is_char() {
            write!(f, "[")?;
        }
        let mut first = true;
        for value in self.to_array().iter() {
            if !first && !V::is_char() {
                write!(f, ", ")?;
            }
            first = false;
            value.fmt_print(f, true)?;
        }
        if !V::is_char() {
            write!(f, "]")
        } else {
            write!(f, "")
        }
    }
}

impl<V: DafnyType> Debug for Sequence<V> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}

// **************
// Immutable maps
// **************

#[derive(Clone)]
pub struct Map<K, V>
where
    K: DafnyTypeEq,
    V: DafnyTypeEq,
{
    data: Rc<HashMap<K, V>>,
}

impl<K: DafnyTypeEq, V: DafnyTypeEq> Default for Map<K, V> {
    fn default() -> Self {
        Map {
            data: Rc::new(HashMap::new()),
        }
    }
}

impl<K: DafnyTypeEq, V: DafnyTypeEq> NontrivialDefault for Map<K, V> {
    fn nontrivial_default() -> Self {
        Self::default()
    }
}

impl<K: DafnyTypeEq, V: DafnyTypeEq> Hash for Map<K, V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.len().hash(state); // Worst performance for things that are not hashable like maps
    }
}

impl<K, V> PartialEq<Map<K, V>> for Map<K, V>
where
    K: DafnyTypeEq,
    V: DafnyTypeEq,
{
    fn eq(&self, other: &Map<K, V>) -> bool {
        if self.data.len() != other.data.len() {
            return false;
        }
        for (k, v) in self.data.iter() {
            if other.data.get(k) != Some(v) {
                return false;
            }
        }
        return true;
    }
}

impl<K: DafnyTypeEq, V: DafnyTypeEq> Eq for Map<K, V> {}

impl<K: DafnyTypeEq, V: DafnyTypeEq> Map<K, V> {
    pub fn new_empty() -> Map<K, V> {
        Map {
            data: Rc::new(HashMap::new()),
        }
    }
    pub fn from_array(values: &Vec<(K, V)>) -> Map<K, V> {
        Self::from_iterator(values.iter().map(|(k, v)| (k.clone(), v.clone())))
    }
    pub fn from_iterator<I>(data: I) -> Map<K, V>
    where
        I: Iterator<Item = (K, V)>,
    {
        let mut result: HashMap<K, V> = HashMap::new();
        for (k, v) in data {
            result.insert(k, v);
        }
        Self::from_hashmap_owned(result)
    }
    pub fn from_hashmap_owned(values: HashMap<K, V>) -> Map<K, V> {
        Map {
            data: Rc::new(values),
        }
    }
    pub fn to_hashmap_owned<K2, V2>(
        &self,
        converter_k: fn(&K) -> K2,
        converter_v: fn(&V) -> V2,
    ) -> HashMap<K2, V2>
    where
        K2: Eq + std::hash::Hash,
        V2: Clone,
    {
        let mut result: HashMap<K2, V2> = HashMap::new();
        for (k, v) in self.data.iter() {
            result.insert(converter_k(k), converter_v(v));
        }
        result
    }
    pub fn cardinality_usize(&self) -> usize {
        self.data.len()
    }
    pub fn cardinality(&self) -> DafnyInt {
        DafnyInt::from_usize(self.cardinality_usize())
    }
    pub fn contains(&self, key: &K) -> bool {
        self.data.contains_key(key)
    }
    pub fn get_or_none(&self, key: &K) -> Option<V> {
        self.data.get(key).cloned()
    }
    // Dafny will normally guarantee that the key exists.
    pub fn get(&self, key: &K) -> V {
        self.data[key].clone()
    }
    pub fn merge(&self, other: &Map<K, V>) -> Map<K, V> {
        if other.cardinality_usize() == 0 {
            return self.clone();
        }
        if self.cardinality_usize() == 0 {
            return other.clone();
        }
        let mut new_data = (*other.data).clone();
        // Overriding self's keys with other's keys if there are some.
        for (k, v) in self.data.iter() {
            if !other.contains(k) {
                new_data.insert(k.clone(), v.clone());
            }
        }
        Self::from_hashmap_owned(new_data)
    }
    pub fn subtract(&self, keys: &Set<K>) -> Self {
        if keys.cardinality_usize() == 0 {
            return self.clone();
        }
        let mut result: HashMap<K, V> = HashMap::new();
        for (k, v) in self.data.iter() {
            if !keys.contains(k) {
                result.insert(k.clone(), v.clone());
            }
        }
        Self::from_hashmap_owned(result)
    }

    pub fn from_hashmap<K2, V2>(
        map: &HashMap<K2, V2>,
        converter_k: fn(&K2) -> K,
        converter_v: fn(&V2) -> V,
    ) -> Map<K, V>
    where
        K: DafnyTypeEq,
        V: DafnyTypeEq,
        K2: Eq + Hash,
        V2: Clone,
    {
        let mut result: HashMap<K, V> = HashMap::new();
        for (k, v) in map.iter() {
            result.insert(converter_k(k), converter_v(v));
        }
        Map {
            data: Rc::new(result),
        }
    }
    pub fn keys(&self) -> Set<K> {
        let mut result: HashSet<K> = HashSet::new();
        for (k, _) in self.data.iter() {
            result.insert(k.clone());
        }
        Set::from_hashset_owned(result)
    }
    pub fn values(&self) -> Set<V> {
        let mut result: Vec<V> = Vec::new();
        for (_, v) in self.data.iter() {
            result.push(v.clone());
        }
        Set::from_array(&result)
    }

    pub fn update_index(&self, index: &K, value: &V) -> Self {
        let mut result = self.data.as_ref().clone();
        result.insert(index.clone(), value.clone());
        Map::from_hashmap_owned(result)
    }

    pub fn update_index_owned(&self, index: K, value: V) -> Self {
        let mut result = self.data.as_ref().clone();
        result.insert(index, value);
        Map::from_hashmap_owned(result)
    }

    pub fn iter_raw(&self) -> std::collections::hash_map::Iter<'_, K, V> {
        self.data.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = K> + '_ {
        self.data.iter().map(|(k, _v)| k).cloned()
    }
}

impl<K: DafnyTypeEq> Map<K, DafnyInt> {
    pub fn as_dafny_multiset(&self) -> Multiset<K> {
        Multiset::from_hashmap(&self.data)
    }
}

pub struct MapBuilder<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    data: HashMap<K, V>,
}

impl<K, V> MapBuilder<K, V>
where
    K: DafnyTypeEq,
    V: DafnyTypeEq,
{
    pub fn new() -> MapBuilder<K, V> {
        MapBuilder {
            data: HashMap::new(),
        }
    }
    pub fn add(&mut self, key: &K, value: &V) {
        // Dafny will prove that overriding has the same value anyway
        self.data.insert(key.clone(), value.clone());
    }
    pub fn build(self) -> Map<K, V> {
        Map::from_hashmap_owned(self.data)
    }
}

impl<K, V> DafnyPrint for Map<K, V>
where
    K: DafnyTypeEq,
    V: DafnyTypeEq,
{
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        f.write_str("map[")?;
        let mut first = true;
        for (k, v) in self.data.iter() {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            k.fmt_print(f, in_seq)?;
            f.write_str(" := ")?;
            v.fmt_print(f, in_seq)?;
        }
        f.write_str("}")
    }
}

impl<K, V> Debug for Map<K, V>
where
    K: DafnyTypeEq,
    V: DafnyTypeEq,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}

// **************
// Immutable sets
// **************

#[derive(Clone)]
pub struct Set<V: DafnyTypeEq> {
    data: Rc<HashSet<V>>,
}

impl<T> Default for Set<T>
where
    T: DafnyTypeEq,
{
    fn default() -> Self {
        Self::new_empty()
    }
}
impl<T: DafnyTypeEq> NontrivialDefault for Set<T> {
    fn nontrivial_default() -> Self {
        Self::default()
    }
}

impl<V> PartialEq<Set<V>> for Set<V>
where
    V: DafnyTypeEq,
{
    fn eq(&self, other: &Set<V>) -> bool {
        // 1. Same cardinality
        // 2. All the elements of self are in the other
        if self.cardinality_usize() != other.cardinality_usize() {
            false
        } else {
            for value in self.data.iter() {
                if !other.contains(value) {
                    return false;
                }
            }
            for value in other.data.iter() {
                if !self.contains(value) {
                    return false;
                }
            }
            true
        }
    }
}

impl<T: DafnyTypeEq> PartialOrd for Set<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Partial ordering is inclusion
        if self.cardinality_usize() <= other.cardinality_usize() {
            for value in self.data.iter() {
                if !other.contains(value) {
                    return None;
                }
            }
            if self.cardinality_usize() == other.cardinality_usize() {
                Some(Ordering::Equal)
            } else {
                Some(Ordering::Less)
            }
        } else {
            for value in other.data.iter() {
                if !self.contains(value) {
                    return None;
                }
            }
            Some(Ordering::Greater)
        }
    }
}

impl<V: DafnyTypeEq> Set<V> {
    pub fn new_empty() -> Set<V> {
        Self::from_hashset_owned(HashSet::new())
    }
    pub fn from_array(array: &Vec<V>) -> Set<V> {
        Self::from_iterator(array.iter().map(|v| v.clone()))
    }
    pub fn from_iterator<I>(data: I) -> Set<V>
    where
        I: Iterator<Item = V>,
    {
        let mut set: HashSet<V> = HashSet::new();
        for value in data {
            set.insert(value);
        }
        Self::from_hashset_owned(set)
    }
    pub fn from_sequence(data: &Rc<Sequence<V>>) -> Set<V> {
        Self::from_array(data.to_array().borrow())
    }
    pub fn from_hashset_owned(hashset: HashSet<V>) -> Set<V> {
        Set {
            data: Rc::new(hashset),
        }
    }
    pub fn cardinality_usize(&self) -> usize {
        self.data.len()
    }
    pub fn cardinality(&self) -> DafnyInt {
        DafnyInt::from_usize(self.data.len())
    }
    pub fn contains(&self, value: &V) -> bool {
        self.data.contains(value)
    }
    pub fn merge(self: &Self, other: &Set<V>) -> Set<V> {
        if self.cardinality_usize() == 0 {
            return other.clone();
        }
        if other.cardinality_usize() == 0 {
            return self.clone();
        }
        let mut result = self.data.as_ref().clone();
        // iterate over the other, add only not contained elements
        for value in other.data.iter() {
            if !result.contains(value) {
                result.insert(value.clone());
            }
        }
        Set::from_hashset_owned(result)
    }

    pub fn intersect(self: &Self, other: &Set<V>) -> Set<V> {
        if self.cardinality_usize() == 0 {
            return self.clone();
        }
        if other.cardinality_usize() == 0 {
            return other.clone();
        }
        // Start with an empty vec with capacity the smallest of both sets
        let mut result = HashSet::new();

        // iterate over the other, take only elements in common
        for value in self.data.iter() {
            if other.data.contains(value) {
                result.insert(value.clone());
            }
        }
        Set::from_hashset_owned(result)
    }

    pub fn subtract(&self, other: &Set<V>) -> Set<V> {
        if self.cardinality_usize() == 0 {
            return self.clone();
        }
        if other.cardinality_usize() == 0 {
            return self.clone();
        }
        // Start with a vec the size of the first one
        let mut result = HashSet::new();

        // iterate over the other, take only elements not in second
        for value in self.data.iter() {
            if !other.contains(value) {
                result.insert(value.clone());
            }
        }
        Set::from_hashset_owned(result)
    }

    pub fn disjoint(&self, other: &Set<V>) -> bool {
        if self.cardinality_usize() == 0 {
            return true;
        }
        if other.cardinality_usize() == 0 {
            return true;
        }
        if other.data.len() < self.data.len() {
            // iterate over the other, take only elements not in self
            for value in other.data.iter() {
                if self.contains(value) {
                    return false;
                }
            }
        } else {
            // iterate over the self, take only elements not in other
            for value in self.data.iter() {
                if other.contains(value) {
                    return false;
                }
            }
        }
        true
    }

    pub fn equals(&self, other: &Set<V>) -> bool {
        if self.cardinality_usize() != other.cardinality_usize() {
            return false;
        }
        // iterate over the other, take only elements not in second
        for value in other.data.iter() {
            if !self.contains(value) {
                return false;
            }
        }
        true
    }

    pub fn elements(self: &Self) -> Set<V> {
        self.clone()
    }

    pub fn as_dafny_multiset(&self) -> Multiset<V> {
        Multiset::from_set(self)
    }

    pub fn iter(&self) -> std::collections::hash_set::Iter<'_, V> {
        self.data.iter()
    }

    pub fn peek(&self) -> V {
        self.data.iter().next().unwrap().clone()
    }
}

pub struct SetBuilder<T>
where
    T: Clone + Eq + std::hash::Hash,
{
    data: HashMap<T, bool>,
}

impl<T: DafnyTypeEq> SetBuilder<T> {
    pub fn new() -> SetBuilder<T> {
        SetBuilder {
            data: HashMap::new(),
        }
    }
    pub fn add(&mut self, value: &T) {
        // Dafny will prove that overriding has the same value anyway
        self.data.insert(value.clone(), true);
    }
    pub fn build(self) -> Set<T> {
        // Iterate over all the key values of the hashmap and add them to an array
        let mut result: Vec<T> = Vec::new();
        for (k, _v) in self.data.iter() {
            result.push(k.clone());
        }

        Set::from_array(&result)
    }
}

impl<V: DafnyTypeEq> DafnyPrint for Set<V> {
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        f.write_str("{")?;
        let mut first = true;
        for value in self.data.iter() {
            if !first {
                f.write_str(", ")?;
            }
            first = false;
            value.fmt_print(f, in_seq)?;
        }
        f.write_str("}")
    }
}

impl<V> Debug for Set<V>
where
    V: DafnyTypeEq,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}

// *******************
// Immutable multisets
// *******************

#[derive(Clone)]
pub struct Multiset<V: DafnyTypeEq> {
    pub data: HashMap<V, DafnyInt>,
    pub size: DafnyInt,
}

impl<V: DafnyTypeEq> Multiset<V> {
    pub fn new_empty() -> Multiset<V> {
        Self::from_array(&vec![])
    }
    pub fn get_total(map: &HashMap<V, DafnyInt>) -> DafnyInt {
        let mut total = DafnyInt::zero();
        for (_, v) in map.iter() {
            total = total + v.clone();
        }
        total
    }
    pub fn from_hashmap_owned(map: HashMap<V, DafnyInt>) -> Multiset<V> {
        Multiset {
            size: Self::get_total(&map),
            data: map,
        }
    }
    pub fn from_hashmap(map: &HashMap<V, DafnyInt>) -> Multiset<V> {
        Self::from_hashmap_owned(map.clone())
    }
    pub fn from_array(data: &Vec<V>) -> Multiset<V> {
        Self::from_iterator(data.iter().map(|x| x.clone()))
    }
    pub fn from_iterator<I>(data: I) -> Multiset<V>
    where
        I: Iterator<Item = V>,
    {
        let mut hashmap: HashMap<V, DafnyInt> = HashMap::new();
        let mut total: DafnyInt = DafnyInt::zero();
        for value in data {
            let count = hashmap.entry(value.clone()).or_insert(DafnyInt::zero());
            *count = count.clone() + DafnyInt::one();
            total = total + DafnyInt::one();
        }
        Multiset {
            data: hashmap,
            size: total,
        }
    }
    pub fn from_set(set: &Set<V>) -> Multiset<V> {
        Self::from_iterator(set.data.iter().map(|v| v.clone()))
    }

    pub fn cardinality_usize(&self) -> SizeT {
        self.size.data.to_usize().unwrap()
    }
    pub fn cardinality(&self) -> DafnyInt {
        self.size.clone()
    }
    pub fn contains(&self, value: &V) -> bool {
        self.data.contains_key(value) && self.data.get(value).unwrap() > &DafnyInt::zero()
    }
    pub fn get(&self, value: &V) -> DafnyInt {
        if self.data.contains_key(value) {
            self.data.get(value).unwrap().clone()
        } else {
            DafnyInt::zero()
        }
    }
    pub fn update_count(&self, value: &V, new_count: &DafnyInt) -> Multiset<V> {
        let mut result = self.clone();
        let old_count = self.get(value);
        if new_count == &DafnyInt::zero() {
            result.data.remove(value);
        } else {
            result.data.insert(value.clone(), new_count.clone());
        }
        result.size = self.size.clone() + new_count.clone() - old_count;
        result
    }
    pub fn merge(&self, other: &Multiset<V>) -> Multiset<V> {
        if other.size.is_zero() {
            return self.clone();
        }
        if self.size.is_zero() {
            return other.clone();
        }
        let mut result = self.data.clone();
        for (k, v) in other.data.iter() {
            let old_count = self.get(k);
            let new_count = old_count.clone() + v.clone();
            result.insert(k.clone(), new_count);
        }
        Multiset {
            data: result,
            size: self.size.clone() + other.size.clone(),
        }
    }
    pub fn intersect(&self, other: &Multiset<V>) -> Multiset<V> {
        if other.size.is_zero() {
            return other.clone();
        }
        if self.size.is_zero() {
            return self.clone();
        }
        let mut result = HashMap::<V, DafnyInt>::new();
        let mut total = DafnyInt::zero();
        for (k, other_count) in other.data.iter() {
            let self_count = self.get(k);
            let resulting_count = if self_count < *other_count {
                self_count
            } else {
                other_count.clone()
            };
            if resulting_count.is_positive() {
                result.insert(k.clone(), resulting_count.clone());
                total = total + resulting_count;
            }
        }
        Multiset {
            data: result,
            size: total,
        }
    }
    pub fn subtract(&self, other: &Multiset<V>) -> Multiset<V> {
        if other.size.is_zero() {
            return self.clone();
        }
        if self.size.is_zero() {
            return self.clone();
        }
        let mut result = self.data.clone();
        let mut total = self.size.clone();
        for (k, v) in other.data.iter() {
            let old_count = self.get(k);
            let new_count = old_count.clone() - v.clone();
            if !new_count.is_positive() {
                total = total - old_count.clone();
                result.remove(k);
            } else {
                total = total - v.clone();
                result.insert(k.clone(), new_count);
            }
        }
        Multiset {
            data: result,
            size: total,
        }
    }
    pub fn disjoint(&self, other: &Multiset<V>) -> bool {
        for value in other.data.keys() {
            if self.contains(value) {
                return false;
            }
        }
        true
    }

    pub fn as_dafny_multiset(&self) -> Multiset<V> {
        self.clone()
    }

    pub fn peek(&self) -> V {
        self.data.iter().next().unwrap().0.clone()
    }

    pub fn iter_raw(&self) -> std::collections::hash_map::Iter<'_, V, DafnyInt> {
        self.data.iter()
    }

    pub fn iter(&self) -> impl Iterator<Item = V> + '_ {
        self.data.iter().map(|(k, _v)| k).cloned()
    }
}

impl<T> Default for Multiset<T>
where
    T: DafnyTypeEq,
{
    fn default() -> Self {
        Self::new_empty()
    }
}
impl<T: DafnyTypeEq> NontrivialDefault for Multiset<T> {
    fn nontrivial_default() -> Self {
        Self::default()
    }
}

impl<V: DafnyTypeEq> PartialOrd<Multiset<V>> for Multiset<V> {
    fn partial_cmp(&self, other: &Multiset<V>) -> Option<Ordering> {
        match self.cardinality().cmp(&other.cardinality()) {
            Ordering::Less => {
                for value in other.data.keys() {
                    if !self.contains(value) || self.get(value) > other.get(value) {
                        return None;
                    }
                }
                Some(Ordering::Less)
            }
            Ordering::Equal => {
                for value in self.data.keys() {
                    if self.get(value) != other.get(value) {
                        return None;
                    }
                }
                Some(Ordering::Equal)
            }
            Ordering::Greater => {
                for value in self.data.keys() {
                    if !other.contains(value) || self.get(value) < other.get(value) {
                        return None;
                    }
                }
                Some(Ordering::Greater)
            }
        }
    }
}

impl<V: DafnyTypeEq> DafnyPrint for Multiset<V> {
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        f.write_str("multiset{")?;
        let mut first = true;
        for value in self.data.iter() {
            for _count in 0..value.1.data.to_usize().unwrap() {
                if !first {
                    f.write_str(", ")?;
                }
                first = false;
                value.0.fmt_print(f, in_seq)?;
            }
        }
        f.write_str("}")
    }
}

impl<V> Debug for Multiset<V>
where
    V: DafnyTypeEq,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}

impl<V: DafnyTypeEq> PartialEq<Multiset<V>> for Multiset<V> {
    fn eq(&self, other: &Multiset<V>) -> bool {
        if self.cardinality() != other.cardinality() {
            return false;
        }
        // iterate over the other, take only elements not in second
        for value in other.data.iter() {
            if !self.contains(value.0) || self.get(value.0) != *value.1 {
                return false;
            }
        }
        true
    }
}
impl<V: DafnyTypeEq> Eq for Multiset<V> {}
impl<V: DafnyTypeEq> Hash for Multiset<V> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.cardinality().hash(state);
    }
}

// Define the AsAny trait
pub trait AsAny {
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
impl AsAny for dyn Any {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
pub fn is_instance_of<C: ?Sized + AsAny, U: 'static>(theobject: *const C) -> bool {
    // safety: Dafny won't call this function unless it can guarantee the object is still allocated
    unsafe { &*theobject }
        .as_any()
        .downcast_ref::<U>()
        .is_some()
}

pub fn dafny_rational_to_int(r: &BigRational) -> BigInt {
    euclidian_division(r.numer().clone(), r.denom().clone())
}

pub fn nullable_referential_equality<T: ?Sized>(left: Option<Rc<T>>, right: Option<Rc<T>>) -> bool {
    match (left, right) {
        (Some(l), Some(r)) => Rc::ptr_eq(&l, &r),
        (None, None) => true,
        _ => false,
    }
}

pub fn euclidian_division<A: Signed + Zero + One + Clone + PartialEq>(a: A, b: A) -> A {
    if !a.is_negative() {
        if !b.is_negative() {
            a / b
        } else {
            -(a / -b)
        }
    } else {
        if !b.is_negative() {
            -((-(a + One::one())) / b) - One::one()
        } else {
            (-(a + One::one())) / (-b) + One::one()
        }
    }
}

pub fn euclidian_modulo<A: Signed + Zero + One + Clone + PartialEq>(a: A, b: A) -> A {
    if !a.is_negative() {
        if !b.is_negative() {
            a % b
        } else {
            a % -b
        }
    } else {
        let bp = b.abs();
        let c = (-a) % bp.clone();
        if c == Zero::zero() {
            Zero::zero()
        } else {
            bp - c
        }
    }
}

pub struct IntegerRange<A: Add<Output = A> + One + Ord + Clone> {
    hi: A,
    current: A,
}

impl<A: Add<Output = A> + One + Ord + Clone> Iterator for IntegerRange<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current < self.hi {
            let result = self.current.clone();
            self.current = self.current.clone() + One::one();
            Some(result)
        } else {
            None
        }
    }
}

pub fn integer_range<A: Add<Output = A> + One + Ord + Clone>(
    low: A,
    hi: A,
) -> impl Iterator<Item = A> {
    IntegerRange { hi, current: low }
}

pub struct IntegerRangeDown<A: Sub<Output = A> + One + Ord + Clone> {
    current: A,
    low: A,
}

impl<A: Sub<Output = A> + One + Ord + Clone> Iterator for IntegerRangeDown<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current > self.low {
            self.current = self.current.clone() - One::one();
            Some(self.current.clone())
        } else {
            None
        }
    }
}

pub fn integer_range_down<A: Sub<Output = A> + One + Ord + Clone>(
    hi: A,
    low: A,
) -> impl Iterator<Item = A> {
    IntegerRangeDown { current: hi, low }
}

// Unbounded versions

pub struct IntegerRangeUnbounded<A: Add<Output = A> + One + Clone> {
    current: A,
}
impl<A: Add<Output = A> + One + Clone> Iterator for IntegerRangeUnbounded<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        let result = self.current.clone();
        self.current = self.current.clone() + One::one();
        Some(result)
    }
}
pub fn integer_range_unbounded<A: Add<Output = A> + One + Clone>(
    low: A,
) -> impl Iterator<Item = A> {
    IntegerRangeUnbounded { current: low }
}

pub struct IntegerRangeDownUnbounded<A: Sub<Output = A> + One + Clone> {
    current: A,
}

impl<A: Sub<Output = A> + One + Clone> Iterator for IntegerRangeDownUnbounded<A> {
    type Item = A;

    fn next(&mut self) -> Option<Self::Item> {
        self.current = self.current.clone() - One::one();
        Some(self.current.clone())
    }
}

pub fn integer_range_down_unbounded<A: Sub<Output = A> + One + Clone>(
    hi: A,
) -> impl Iterator<Item = A> {
    IntegerRangeDownUnbounded { current: hi }
}

pub struct LazyFieldWrapper<A>(pub Lazy<A, Box<dyn 'static + FnOnce() -> A>>);

impl<A: PartialEq> PartialEq for LazyFieldWrapper<A> {
    fn eq(&self, other: &Self) -> bool {
        self.0.deref() == other.0.deref()
    }
}

impl<A: Default + 'static> Default for LazyFieldWrapper<A> {
    fn default() -> Self {
        Self(Lazy::new(Box::new(A::default)))
    }
}

impl<A: DafnyPrint> DafnyPrint for LazyFieldWrapper<A> {
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        self.0.deref().fmt_print(f, in_seq)
    }
}

// Convert the DafnyPrint above into a macro so that we can create it for functions of any input arity
macro_rules! dafny_print_function {
    ($($n:tt)*) => {
        impl <B, $($n),*> DafnyPrint for Rc<dyn Fn($($n),*) -> B> {
            fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
                write!(f, "<function>")
            }
        }
    }
}
// Now create a loop like impl_tuple_print_loop so that we can create functions up to size 32
macro_rules! dafny_print_function_loop {
    ($first:ident $($rest:ident)*) => {
        dafny_print_function_loop! { $($rest)* }
        dafny_print_function! { $first $($rest)* }
    };
    () => {
    }
}
// Emit functions till 32 parameters
dafny_print_function_loop! { A1 A2 A3 A4 A5 A6 A7 A8 A9 A10 A11 A12 A13 A14 A15 A16
A17 A18 A19 A20 A21 A22 A23 A24 A25 A26 A27 A28 A29 A30 A31 A32 }

pub struct FunctionWrapper<A: ?Sized>(pub A);
impl<A> DafnyPrint for FunctionWrapper<A> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "<function>")
    }
}

impl<A: Clone> Clone for FunctionWrapper<A> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<A: ?Sized> PartialEq for FunctionWrapper<Rc<A>> {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

pub struct DafnyPrintWrapper<T>(pub T);
impl<T: DafnyPrint> Display for DafnyPrintWrapper<&T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt_print(f, false)
    }
}

// from gazebo
#[inline]
pub unsafe fn transmute_unchecked<A, B>(x: A) -> B {
    assert_eq!(std::mem::size_of::<A>(), std::mem::size_of::<B>());
    debug_assert_eq!(0, (&x as *const A).align_offset(std::mem::align_of::<B>()));
    let b = std::ptr::read(&x as *const A as *const B);
    std::mem::forget(x);
    b
}

pub trait DafnyPrint {
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result;

    // Vec<char> gets special treatment so we carry that information here
    #[inline]
    fn is_char() -> bool {
        false
    }
}

impl<T> DafnyPrint for *const T {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "<{} object>", std::any::type_name::<T>())
    }
}

macro_rules! impl_print_display {
    ($name:ty) => {
        impl DafnyPrint for $name {
            fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
                std::fmt::Display::fmt(&self, f)
            }
        }
    };
}

impl_print_display! { String }
impl_print_display! { bool }
impl_print_display! { u8 }
impl_print_display! { u16 }
impl_print_display! { u32 }
impl_print_display! { u64 }
impl_print_display! { u128 }
impl_print_display! { i8 }
impl_print_display! { i16 }
impl_print_display! { i32 }
impl_print_display! { i64 }
impl_print_display! { i128 }
impl_print_display! { usize }

impl DafnyPrint for f32 {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "{:.1}", self)
    }
}

impl DafnyPrint for f64 {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "{:.1}", self)
    }
}

impl DafnyPrint for () {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "()")
    }
}

#[derive(Clone)]
pub struct DafnyCharUTF16(pub u16);
pub type DafnyStringUTF16 = Sequence<DafnyCharUTF16>;

impl DafnyPrint for DafnyCharUTF16 {
    #[inline]
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        let real_char = char::decode_utf16(vec![self.clone()].iter().map(|v| v.0))
            .map(|r| r.map_err(|e| e.unpaired_surrogate()))
            .collect::<Vec<_>>()[0];
        let rendered_char = match real_char {
            Ok(c) => c,
            Err(e) => {
                return write!(f, "Invalid UTF-16 code point: {}", e);
            }
        };

        if in_seq {
            write!(f, "{}", rendered_char)
        } else {
            write!(f, "'{}'", rendered_char)
        }
    }

    #[inline]
    fn is_char() -> bool {
        true
    }
}

impl Debug for DafnyCharUTF16 {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}

impl PartialEq<DafnyCharUTF16> for DafnyCharUTF16 {
    fn eq(&self, other: &DafnyCharUTF16) -> bool {
        self.0 == other.0
    }
}
impl Eq for DafnyCharUTF16 {}
impl Hash for DafnyCharUTF16 {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl PartialOrd<DafnyCharUTF16> for DafnyCharUTF16 {
    fn partial_cmp(&self, other: &DafnyCharUTF16) -> Option<Ordering> {
        (self.0).partial_cmp(&other.0)
    }
}

#[derive(Clone)]
pub struct DafnyChar(pub char);
pub type DafnyString = Sequence<DafnyChar>;

impl DafnyPrint for DafnyChar {
    #[inline]
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        if in_seq {
            write!(f, "{}", self.0)
        } else {
            write!(f, "'{}'", self.0)
        }
    }

    #[inline]
    fn is_char() -> bool {
        true
    }
}

impl Debug for DafnyChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}

impl PartialEq<DafnyChar> for DafnyChar {
    fn eq(&self, other: &DafnyChar) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<DafnyChar> for DafnyChar {
    fn partial_cmp(&self, other: &DafnyChar) -> Option<Ordering> {
        (self.0 as u32).partial_cmp(&(other.0 as u32))
    }
}
impl Eq for DafnyChar {}
impl Hash for DafnyChar {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state)
    }
}

impl<T: DafnyPrint> DafnyPrint for Option<T> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        match self {
            Some(x) => x.fmt_print(f, false),
            None => write!(f, "null"),
        }
    }
}

impl DafnyPrint for BigInt {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "{}", self)
    }
}

fn divides_a_power_of_10(mut i: BigInt) -> (bool, BigInt, usize) {
    let one: BigInt = One::one();

    let mut factor = one.clone();
    let mut log10 = 0;

    let zero = Zero::zero();
    let ten = BigInt::from_i32(10).unwrap();

    if i <= zero {
        return (false, factor, log10);
    }

    while i.is_multiple_of(&ten) {
        i /= BigInt::from_i32(10).unwrap();
        log10 += 1;
    }

    let two = BigInt::from_i32(2).unwrap();
    let five = BigInt::from_i32(5).unwrap();

    while i.is_multiple_of(&five) {
        i /= &five;
        factor *= &two;
        log10 += 1;
    }

    while i.is_multiple_of(&two) {
        i /= &two;
        factor *= &two;
        log10 += 1;
    }

    (i == one, factor, log10)
}

impl DafnyPrint for BigRational {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        if self.denom() == &One::one() || self.numer() == &Zero::zero() {
            write!(f, "{}.0", self.numer())
        } else {
            let (divides, factor, log10) = divides_a_power_of_10(self.denom().clone());
            if divides {
                let mut num = self.numer().clone();
                num *= factor;

                if num.is_negative() {
                    write!(f, "-")?;
                    num = -num;
                }

                let digits = num.to_string();

                if log10 < digits.len() {
                    let digit_count = digits.len() - log10;
                    write!(f, "{}", &digits[..digit_count])?;
                    write!(f, ".")?;
                    write!(f, "{}", &digits[digit_count..])
                } else {
                    let z = log10 - digits.len();
                    write!(f, "0.")?;
                    for _ in 0..z {
                        write!(f, "0")?;
                    }
                    write!(f, "{}", digits)
                }
            } else {
                write!(f, "({}.0 / {}.0)", self.numer(), self.denom())
            }
        }
    }
}

impl<T: DafnyPrint> DafnyPrint for Rc<T> {
    fn fmt_print(&self, f: &mut Formatter<'_>, in_seq: bool) -> std::fmt::Result {
        self.as_ref().fmt_print(f, in_seq)
    }
}

impl<T: DafnyPrint> DafnyPrint for Vec<T> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        if !T::is_char() {
            write!(f, "[")?;
        }

        for (i, item) in self.iter().enumerate() {
            if !T::is_char() {
                if i > 0 {
                    write!(f, ", ")?;
                }

                item.fmt_print(f, false)?;
            } else {
                item.fmt_print(f, true)?;
            }
        }

        if !T::is_char() {
            write!(f, "]")
        } else {
            Ok(())
        }
    }
}

impl<T: DafnyPrint> DafnyPrint for RefCell<T> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        self.borrow().fmt_print(f, _in_seq)
    }
}

impl<T: DafnyPrint> DafnyPrint for HashSet<T> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "{{")?;

        let mut i = 0;

        for item in self.iter() {
            if i > 0 {
                write!(f, ", ")?;
            }

            item.fmt_print(f, false)?;

            i += 1;
        }

        write!(f, "}}")
    }
}

pub fn char_lt(left: char, right: char) -> bool {
    let left_code = left as u32;
    let right_code = right as u32;

    left_code < right_code
}

pub fn string_of(s: &str) -> DafnyString {
    s.chars()
        .map(|v| DafnyChar(v))
        .collect::<Sequence<DafnyChar>>()
}

pub fn string_utf16_of(s: &str) -> DafnyStringUTF16 {
    Sequence::from_array_owned(s.encode_utf16().map(|v| DafnyCharUTF16(v)).collect())
}

macro_rules! impl_tuple_print {
    ($($items:ident)*) => {
        impl <$($items,)*> DafnyPrint for ($($items,)*)
        where
            $($items: DafnyPrint,)*
        {
            #[allow(unused_assignments)]
            fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
                #[allow(non_snake_case)]
                let ($($items,)*) = self;

                write!(f, "(")?;

                let mut i = 0;

                $(
                    if (i > 0) {
                        write!(f, ", ")?;
                    }

                    $items.fmt_print(f, false)?;

                    i += 1;
                )*

                write!(f, ")")
            }
        }
    };
}

macro_rules! impl_tuple_print_loop {
    () => {};
    ($first:ident $($rest:ident)*) => {
        impl_tuple_print_loop! { $($rest)* }
        impl_tuple_print! { $first $($rest)* }
    };
}

// 32 elements ought to be enough for everyone
impl_tuple_print_loop! {
    A0 A1 A2 A3 A4 A5 A6 A7 A8 A9 A10
    A11 A12 A13 A14 A15 A16 A17 A18 A19 A20
    A21 A22 A23 A24 A25 A26 A27 A28 A29 A30
    A31
}

// seq!(1, 2, 3) is rewritten to Sequence::from_array_owned(vec!(1, 2, 3))
#[macro_export]
macro_rules! seq {
    ($($x:expr),*) => {
        $crate::Sequence::from_array_owned(vec![$($x), *])
    }
}

#[macro_export]
macro_rules! set {
    ($($x:expr), *) => {
        {
            // No warning about this variable not needing to be mut in the case of an empty set
            #[allow(unused_mut)]
            let mut temp_hash = ::std::collections::HashSet::new();
            $(
                temp_hash.insert($x);
            )*
            $crate::Set::from_hashset_owned(temp_hash)
        }
    }
}

#[macro_export]
macro_rules! multiset {
    ($($x:expr), *) => {
        {
            #[allow(unused_mut)]
            let mut temp_hash = ::std::collections::HashMap::new();
            #[allow(unused_mut)]
            let mut total_size: usize = 0;
            $( {
                #[allow(unused_mut)]
                let mut entry = temp_hash.entry($x).or_insert($crate::DafnyInt::from(0));
                *entry = (*entry).clone() + $crate::DafnyInt::from(1);
                total_size += 1;
              }
            )*
            $crate::Multiset {
                data: temp_hash,
                size: $crate::DafnyInt::from(total_size),
            }
        }
    }
}

// we enable the syntax map![k1 => v1, k2 => v2]
#[macro_export]
macro_rules! map {
    ($($k:expr => $v:expr), *) => {
        {
            #[allow(unused_mut)]
            let mut temp_hash = ::std::collections::HashMap::new();
            $(
                temp_hash.insert($k.clone(), $v.clone());
            )*
            $crate::Map::from_hashmap_owned(temp_hash)
        }
    }
}

#[macro_export]
macro_rules! int {
    ($x:expr) => {
        $crate::DafnyInt::from($x)
    };
}

//////////
// Arrays
//////////

macro_rules! ARRAY_GETTER_LENGTH0 {
    () => {
        #[inline]
        pub fn length0(&self) -> DafnyInt {
            DafnyInt::from(self.length0_usize())
        }
        #[inline]
        pub fn length0_usize(&self) -> usize {
            self.data.len()
        }
    }
}
macro_rules! ARRAY_GETTER_LENGTH {
    ($field: ident, $field_usize: ident) => {
        #[inline]
        pub fn $field(&self) -> DafnyInt {
            $crate::DafnyInt::from(self.$field_usize())
        }
        #[inline]
        pub fn $field_usize(&self) -> usize {
            self.$field
        }
    }
}

// An 1-dimensional Dafny array is a zero-cost abstraction over a pointer on a native array
#[macro_export]
macro_rules! array {
    ($($x:expr), *) => {
        array::from_native(Box::new([$($x), *]))
    }
}

macro_rules! ARRAY_INIT {
    {$length: ident, $inner: expr} => {
        $crate::array::initialize_box_usize($length, {
            Rc::new(move |_| { $inner })
        })
    }
}

macro_rules! ARRAY_INIT_INNER {
    ($length: ident) => {
        $crate::array::placebos_box_usize::<T>($length)
    }
}

// ARRAY_DATA_TYPE(length0, length1, length2) will return
// Box<[Box<[Box<[T]>]>]>
macro_rules! ARRAY_DATA_TYPE {
    ($length:ident) => {
        Box<[T]>
    };
    ($length:ident, $($rest_length:ident),+) => {
        Box<[ARRAY_DATA_TYPE!($($rest_length),+)]>
    };
}

// Macro to generate generalizations of the function placebos_usize to higher-dimensions arrays

#[macro_export]
macro_rules! INIT_ARRAY_DATA {
    // Handle the innermost array initialization
    ($ArrayType:ident, $last_length:ident) => {
        ARRAY_INIT_INNER!($last_length)
    };
    // Handle recursive array initialization for multiple dimensions
    ($ArrayType:ident, $first_length:ident, $($rest_length:ident),+) => {
        ARRAY_INIT!($first_length, INIT_ARRAY_DATA!($ArrayType, $($rest_length),+))
    };
}

macro_rules! ARRAY_METHODS {
    // Accepts any number of length identifiers
    ($ArrayType:ident, $length0: ident, $($length:ident),+) => {
        pub fn placebos_usize(
            $length0: usize,
            $($length: usize),+
        ) -> *mut $ArrayType<$crate::MaybeUninit<T>> {
            Box::into_raw(Box::new($ArrayType {
                $($length: $length),+,
                data: INIT_ARRAY_DATA!($ArrayType, $length0, $($length),+),
            }))
        }

        pub fn placebos(
            $length0: &$crate::DafnyInt,
            $($length: &$crate::DafnyInt),+
        ) -> *mut $ArrayType<$crate::MaybeUninit<T>> {
            Self::placebos_usize(
                $length0.as_usize(),
                $($length.as_usize()),+
            )
        }

        // Once all the elements have been initialized, transform the signature of the pointer
        pub fn construct(p: *mut $ArrayType<$crate::MaybeUninit<T>>) -> *mut $ArrayType<T> {
            unsafe { std::mem::transmute(p) }
        }
        // Once all the elements have been initialized, transform the signature of the pointer
        pub fn construct_rcmut(p: $crate::Object<$ArrayType<MaybeUninit<T>>>) -> Object<$ArrayType<T>> {
            unsafe { std::mem::transmute(p) }
        }
    };
}


macro_rules! ARRAY_STRUCT {
    ($ArrayType:ident, $length0: ident, $($length:ident),+) => {
        pub struct $ArrayType<T> {
            $($length: usize),+,
            data: ARRAY_DATA_TYPE!($length0, $($length),+),
        }
    }
}

macro_rules! ARRAY_TO_VEC_LOOP {
    (@inner $self: ident, $tmp: ident, $data: expr) => {
        $tmp.push($data.clone());
    };
    (@for $self: ident, $tmp: ident, $data: expr, $length_usize: ident $(, $rest_length_usize: ident)*) => {
        for i in 0..$self.$length_usize() {
            ARRAY_TO_VEC_LOOP!(@inner $self, $tmp, $data[i] $(, $rest_length_usize)*);
        }
    };
    (@inner $self: ident, $outerTmp: ident, $data: expr $(, $rest_length_usize: ident)*) => {
        {
            let mut tmp = Vec::new();
            ARRAY_TO_VEC_LOOP!(@for $self, tmp, $data $(, $rest_length_usize)*);
            $outerTmp.push(tmp);
        }
    };
    
    ($self: ident, $data: expr $(, $rest_length_usize: ident)*) => {
        {
            let mut tmp = Vec::new();
            ARRAY_TO_VEC_LOOP!(@for $self, tmp, $data $(, $rest_length_usize)*);
            tmp
        }
    };
}

macro_rules! ARRAY_TO_VEC_TYPE {
    ($length0: ident) => {
        Vec<T>
    };
    ($length0: ident $(, $res_length: ident)*) => {
        Vec<ARRAY_TO_VEC_TYPE!($($res_length), *)>
    }
}

macro_rules! ARRAY_TO_VEC {
    ($length0_usize: ident $(, $res_length_usize: ident)*) => {
        pub fn to_vec(&self) -> ARRAY_TO_VEC_TYPE!($length0_usize, $($res_length_usize),*) {
            ARRAY_TO_VEC_LOOP!(self, self.data, $length0_usize, $($res_length_usize),*)
        }
    }
}

macro_rules! ARRAY_LENGTHS {
    () => {

    };
    (($length0: ident, $length0_usize: ident) $(, $rest: tt)*) => {
        ARRAY_GETTER_LENGTH0!();
        ARRAY_LENGTHS!($(, $rest)*);
    };
    (, ($length: ident, $length_usize: ident) $(, $rest: tt)*) => {
        ARRAY_GETTER_LENGTH!($length, $length_usize);
        ARRAY_LENGTHS!($(, $rest)*);
    }
}

macro_rules! ARRAY_METHODS_WRAPPER {
    ($ArrayType:ident, $(($length:ident, $length_usize: ident)), +) => {
        ARRAY_METHODS!($ArrayType, $($length), +);
    }
}

macro_rules! ARRAY_TO_VEC_WRAPPER {
    ($(($length:ident, $length_usize: ident)), +) => {
        ARRAY_TO_VEC!($($length_usize), +);
    }
}

macro_rules! ARRAY_STRUCT_WRAPPER {
    ($ArrayType:ident, $(($length:ident, $length_usize: ident)), +) => {
        ARRAY_STRUCT!($ArrayType, $($length), +);
    }
}

macro_rules! ARRAY_DEF {
    ($ArrayType:ident, $(($length:ident, $length_usize: ident)), +) => {
        ARRAY_STRUCT_WRAPPER!($ArrayType, $(($length, $length_usize)), +);
        impl<T> $ArrayType<T> {
            ARRAY_LENGTHS!{
                $(($length, $length_usize)), +
            }
            ARRAY_METHODS_WRAPPER!{$ArrayType,
                $(($length, $length_usize)), +
            }
        }
        impl<T: Clone> $ArrayType<T> {
            ARRAY_TO_VEC_WRAPPER!{
                $(($length, $length_usize)), +
            }
        }
    }
}

// Array2 to Array16

ARRAY_DEF!{Array2, 
    (length0, length0_usize),
    (length1, length1_usize)
}

ARRAY_DEF!{Array3, 
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize)
}

ARRAY_DEF!{Array4,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize)
}

ARRAY_DEF!{Array5,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize)
}

ARRAY_DEF!{Array6,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize)
}

ARRAY_DEF!{Array7,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize)
}

ARRAY_DEF!{Array8,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize)
}

ARRAY_DEF!{Array9,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize)
}

ARRAY_DEF!{Array10,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize)
}

ARRAY_DEF!{Array11,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize),
    (length10, length10_usize)
}

ARRAY_DEF!{Array12,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize),
    (length10, length10_usize),
    (length11, length11_usize)
}

ARRAY_DEF!{Array13,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize),
    (length10, length10_usize),
    (length11, length11_usize),
    (length12, length12_usize)
}

ARRAY_DEF!{Array14,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize),
    (length10, length10_usize),
    (length11, length11_usize),
    (length12, length12_usize),
    (length13, length13_usize)
}

ARRAY_DEF!{Array15,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize),
    (length10, length10_usize),
    (length11, length11_usize),
    (length12, length12_usize),
    (length13, length13_usize),
    (length14, length14_usize)
}

ARRAY_DEF!{Array16,
    (length0, length0_usize),
    (length1, length1_usize),
    (length2, length2_usize),
    (length3, length3_usize),
    (length4, length4_usize),
    (length5, length5_usize),
    (length6, length6_usize),
    (length7, length7_usize),
    (length8, length8_usize),
    (length9, length9_usize),
    (length10, length10_usize),
    (length11, length11_usize),
    (length12, length12_usize),
    (length13, length13_usize),
    (length14, length14_usize),
    (length15, length15_usize)
}

pub mod array {
    use super::DafnyInt;
    use num::ToPrimitive;
    use std::mem::MaybeUninit;
    use std::{boxed::Box, rc::Rc, vec::Vec};
    #[inline]
    pub fn from_native<T>(v: Box<[T]>) -> *mut [T] {
        Box::into_raw(v)
    }
    #[inline]
    pub fn from_vec<T>(v: Vec<T>) -> *mut [T] {
        from_native(v.into_boxed_slice())
    }
    pub fn to_vec<T>(v: *mut [T]) -> Vec<T> {
        unsafe { Box::from_raw(v) }.into_vec()
    }
    pub fn initialize_usize<T>(n: usize, initializer: Rc<dyn Fn(usize) -> T>) -> *mut [T] {
        let mut v = Vec::with_capacity(n);
        for i in 0..n {
            v.push(initializer(i));
        }
        from_vec(v)
    }

    pub fn placebos<T>(n: &DafnyInt) -> *mut [MaybeUninit<T>] {
        placebos_usize(n.as_usize())
    }
    pub fn placebos_usize<T>(n: usize) -> *mut [MaybeUninit<T>] {
        Box::into_raw(placebos_box_usize(n))
    }
    pub fn placebos_usize_rcmut<T>(n: usize) -> super::Object<[MaybeUninit<T>]> {
        super::rcmut::array_object_from_box(placebos_box_usize(n))
    }
    // Once all the elements have been initialized, transform the signature of the pointer
    pub fn construct<T>(p: *mut [MaybeUninit<T>]) -> *mut [T] {
        unsafe { std::mem::transmute(p) }
    }
    pub fn construct_rcmut<T>(p: super::Object<[MaybeUninit<T>]>) -> super::Object<[T]> {
        unsafe { std::mem::transmute(p) }
    }

    pub fn placebos_box<T>(n: &DafnyInt) -> Box<[MaybeUninit<T>]> {
        placebos_box_usize(n.to_usize().unwrap())
    }
    pub fn placebos_box_usize<T>(n_usize: usize) -> Box<[MaybeUninit<T>]> {
        // This code is optimized to take a constant time. See:
        // https://users.rust-lang.org/t/allocate-a-boxed-array-of-maybeuninit/110169/7
        std::iter::repeat_with(MaybeUninit::uninit)
            .take(n_usize)
            .collect()
    }

    pub fn initialize<T>(n: &DafnyInt, initializer: Rc<dyn Fn(&DafnyInt) -> T>) -> *mut [T] {
        Box::into_raw(initialize_box(n, initializer))
    }

    pub fn initialize_box<T>(n: &DafnyInt, initializer: Rc<dyn Fn(&DafnyInt) -> T>) -> Box<[T]> {
        initialize_box_usize(n.to_usize().unwrap(), initializer)
    }
    pub fn initialize_box_usize<T>(
        n_usize: usize,
        initializer: Rc<dyn Fn(&DafnyInt) -> T>,
    ) -> Box<[T]> {
        let mut v = Vec::with_capacity(n_usize);
        for i in 0..n_usize {
            v.push(initializer(&int!(i)));
        }
        v.into_boxed_slice()
    }

    #[inline]
    pub fn length_usize<T>(this: *mut [T]) -> usize {
        // safety: Dafny won't call this function unless it can guarantee the array is still allocated
        super::read!(this).len()
    }
    #[inline]
    pub fn length<T>(this: *mut [T]) -> DafnyInt {
        int!(length_usize(this))
    }
    #[inline]
    pub fn get_usize<T: Clone>(this: *mut [T], i: usize) -> T {
        // safety: Dafny won't call this function unless it can guarantee the array is still allocated
        (unsafe { &*this } as &[T])[i].clone()
    }
    #[inline]
    pub fn get<T: Clone>(this: *mut [T], i: &DafnyInt) -> T {
        get_usize(this, i.to_usize().unwrap())
    }
    #[inline]
    pub fn update_usize<T>(this: *mut [T], i: usize, val: T) {
        // safety: Dafny won't call this function unless it can guarantee the array is still allocated
        (unsafe { &mut *this } as &mut [T])[i] = val;
    }
    #[inline]
    pub fn update<T>(this: *mut [T], i: &DafnyInt, val: T) {
        update_usize(this, i.to_usize().unwrap(), val);
    }
}

///////////////////
// Class helpers //
///////////////////
pub fn allocate<T>() -> *mut T {
    let mut this: Box<MaybeUninit<T>> = Box::new(MaybeUninit::uninit());
    let this_ptr = this.as_mut() as *mut MaybeUninit<T> as *mut T;
    Box::into_raw(this); // Make sure this is not dropped
    this_ptr
}
// Generic function to safely deallocate a raw pointer
#[inline]
pub fn deallocate<T: ?Sized>(pointer: *const T) {
    // safety: Dafny won't call this function unless it can guarantee the array is still allocated
    unsafe {
        // Takes ownership of the reference,
        // so that it's deallocated at the end of the method
        let _ = Box::from_raw(pointer as *mut T);
    }
}

impl<T: ?Sized> DafnyPrint for *mut T {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "object")
    }
}

impl<T> NontrivialDefault for *mut T {
    fn nontrivial_default() -> Self {
        0 as *mut T
    }
}
pub struct ExactPool<T: Clone> {
    current: T,
    yielded: bool,
}

// Implement iterator for an exact pool, yielding
impl<T: Clone> Iterator for ExactPool<T> {
    type Item = T;
    fn next(&mut self) -> Option<Self::Item> {
        if self.yielded {
            None
        } else {
            self.yielded = true;
            Some(self.current.clone())
        }
    }
}
pub fn exact_range<T: Clone>(value: T) -> ExactPool<T> {
    ExactPool {
        current: value,
        yielded: false,
    }
}

// Any Dafny trait must require classes extending it to have a method "as_any_mut"
// that can convert the reference from that trait to a reference of Any

// cast is meant to be used on references only, to downcast a trait reference to a class reference
#[macro_export]
macro_rules! cast {
    ($raw:expr, $id:ty) => {
        $crate::modify!($raw)
            .as_any_mut()
            .downcast_mut::<$id>()
            .unwrap() as *mut $id
    };
}

// 'is' is meant to be used on references only, to check if a trait reference is a class reference
#[macro_export]
macro_rules! is {
    ($raw:expr, $id:ty) => {
        $crate::modify!($raw)
            .as_any_mut()
            .downcast_mut::<$id>()
            .is_some()
    };
}

// cast_any is meant to be used on references only, to convert any references (classes or traits)*
// to an Any reference trait
#[macro_export]
macro_rules! cast_any {
    ($raw:expr) => {
        $crate::modify!($raw).as_any_mut()
    };
}

// When initializing an uninitialized field for the first time,
// we ensure we don't drop the previous content
// This is problematic if the same field is overwritten multiple times
/// In that case, prefer to use update_uninit
#[macro_export]
macro_rules! update_field_nodrop {
    ($ptr:expr, $field:ident, $value:expr) => {
        $crate::update_nodrop!((*$ptr).$field, $value)
    };
}

// When initializing an uninitialized field for the first time,
// we ensure we don't drop the previous content
#[macro_export]
macro_rules! update_nodrop {
    ($ptr:expr, $value:expr) => {
        // safety: Dafny won't call this function unless it can guarantee the value at the address was not
        // yet initialized, so that not dropping it won't create a memory leak
        unsafe { ::std::ptr::addr_of_mut!($ptr).write($value) }
    }
}

// Given a class or array pointer, transforms it to a mutable reference
#[macro_export]
macro_rules! modify {
    ($ptr:expr) => {
        // safety: Dafny will only obtain a mutable borrowed address of a pointer if it can ensure the object
        // is still allocated
        (unsafe { &mut *$ptr })
    }
}

// Given a class or array pointer, transforms it to a read-only reference
#[macro_export]
macro_rules! read {
    ($ptr:expr) => {
        // safety: Dafny will only obtain a borrowed address of a pointer if it can ensure the object
        // is still allocated
        (unsafe { &*$ptr })
    }
}

// If the field is guaranteed to be assigned only once, update_field_nodrop is sufficient
#[macro_export]
macro_rules! update_field_uninit {
    ($t:expr, $field:ident, $field_assigned:expr, $value:expr) => {{
        let computed_value = $value;
        if $field_assigned {
            $crate::modify!($t).$field = computed_value;
        } else {
            $crate::update_field_nodrop!($t, $field, computed_value);
            $field_assigned = true;
        }
    }};
}

// Macro to call at the end of the first new; constructor when not every field is guaranteed to be assigned.
#[macro_export]
macro_rules! update_field_if_uninit {
    ($t:expr, $field:ident, $field_assigned:expr, $value:expr) => {{
        let computed_value = $value;
        if !$field_assigned {
            $crate::update_field_nodrop!($t, $field, computed_value);
            $field_assigned = true;
        }
    }};
}

/////////////////
// Reference-counted classes mode
/////////////////

pub struct Object<T: ?Sized>(pub Option<rcmut::RcMut<T>>);

impl<T: ?Sized> Clone for Object<T> {
    fn clone(&self) -> Self {
        Object(self.0.clone())
    }
}

impl<T: ?Sized> Debug for Object<T> {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        self.fmt_print(f, false)
    }
}
impl <T: ?Sized> DafnyPrint for Object<T> {
    fn fmt_print(&self, f: &mut Formatter<'_>, _in_seq: bool) -> std::fmt::Result {
        write!(f, "<object>")
    }
}

impl <T: ?Sized> PartialEq for Object<T> {
    fn eq(&self, other: &Self) -> bool {
        if let Some(p) = &self.0 {
            if let Some(q) = &other.0 {
                // To compare addresses, we need to ensure we only compare thin pointers
                // https://users.rust-lang.org/t/comparing-addresses-between-fat-and-thin-pointers/89008
                ::std::ptr::eq(p.as_ref().get() as *const (), q.as_ref().get() as *const ())
            } else {
                false
            }
        } else {
            false
        }
    }
}

impl <T: ?Sized> std::hash::Hash for Object<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(p) = &self.0 {
            p.as_ref().get().hash(state);
        } else {
            0.hash(state);
        }
    }
}

#[macro_export]
macro_rules! cast_object {
    ($raw:expr, $id:ty) => {
        unsafe {
            let res: $crate::Object<$id> = 
            $crate::Object(Some(::std::rc::Rc::from_raw(
                ::std::rc::Rc::into_raw($raw.0.unwrap()) as _)));
            res
        }
    };
}

// Returns an object whose fields are yet initialized. Only use update_field_uninit_rcmut  and update_field_if_uninit_rcmut to initialize fields.
pub fn allocate_rcmut<T>() -> Object<T> {
    unsafe { mem::transmute(object::new::<MaybeUninit<T>>(MaybeUninit::uninit())) }
}

pub struct AllocationTracker {
    allocations: Vec<Weak<dyn Any>>
}

pub fn allocate_rcmut_track<T: 'static>(allocation_tracker: &mut AllocationTracker) -> Object<T> {
    let res = allocate_rcmut::<T>();
    allocation_tracker.allocations.push(Rc::<UnsafeCell<T>>::downgrade(&res.0.clone().unwrap()));
    res
}

pub fn is_instance_of_rcmut<T: ?Sized + AsAny + 'static, U: 'static>(theobject: Object<T>) -> bool {
    // safety: Dafny won't call this function unless it can guarantee the object is still allocated
    unsafe { 
        rcmut::borrow(&theobject.0.unwrap()).as_any().downcast_ref::<U>().is_some()
    }
}

// Equivalent of update_field_nodrop but for rcmut
#[macro_export]
macro_rules! update_field_nodrop_rcmut {
    ($ptr:expr, $field: ident, $value:expr) => {
        $crate::update_nodrop_rcmut!(($crate::rcmut::borrow_mut(&mut $ptr.0.clone().unwrap())).$field, $value)
    };
}

// Equivalent of update_nodrop but for rcmut
#[macro_export]
macro_rules! update_nodrop_rcmut {
    ($ptr:expr, $value:expr) => {
        unsafe { unsafe { ::std::ptr::addr_of_mut!($ptr).write($value) } }
    };
}

// Equivalent of update_field_if_uninit but for rcmut
#[macro_export]
macro_rules! update_field_if_uninit_rcmut {
    ($t:expr, $field:ident, $field_assigned:expr, $value:expr) => {{
        let computed_value = $value;
        if !$field_assigned {
            $crate::update_field_nodrop_rcmut!($t, $field, computed_value);
            $field_assigned = true;
        }
    }};
}

// Equivalent of update_field_uninit but for rcmut
#[macro_export]
macro_rules! update_field_uninit_rcmut {
    ($t:expr, $field:ident, $field_assigned:expr, $value:expr) => {{
        let computed_value = $value;
        if $field_assigned {
            $crate::md!($t).$field = computed_value;
        } else {
            $crate::update_field_nodrop_rcmut!($t, $field, computed_value);
            $field_assigned = true;
        }
    }};
}

// Equivalent of modify but for rcmut
#[macro_export]
macro_rules! md {
    ($x:expr) => {
        unsafe { $crate::rcmut::borrow_mut(&mut $x.0.unwrap()) }
    };
}

// Equivalent of read but for rcmut
#[macro_export]
macro_rules! rd {
    ($x:expr) => {
        unsafe { $crate::rcmut::borrow(& $x.0.unwrap()) }
    };
}

// Count the number of references to the given object
#[macro_export]
macro_rules! refcount {
    ($x:expr) => {
        Rc::strong_count(unsafe { rcmut::as_rc(& $x.0.unwrap()) })
    };
}

pub mod object {
    pub fn new<T>(val: T) -> crate::Object<T> {
        crate::Object(Some(crate::rcmut::new(val)))
    }
}

// Inspired from https://crates.io/crates/rcmut
pub mod rcmut {
    use std::cell::UnsafeCell;
    use std::mem::{self, MaybeUninit};
    use std::rc::Rc;
    use std::sync::Arc;

    pub fn array_object_from_rc<T>(data: Rc<[T]>) -> crate::Object<[T]> {
        crate::Object(Some(unsafe { crate::rcmut::from_rc(data) }))
    }
    pub fn array_object_from_box<T>(data: Box<[T]>) -> crate::Object<[T]> {
        let data: Rc<[T]> = data.into();
        crate::Object(Some(unsafe { crate::rcmut::from_rc(data) }))
    }
    pub struct Array<T> {
        pub data: Box<[T]>,
    }
    impl<T> Array<T> {
        pub fn new(data: Box<[T]>) -> crate::Object<Array<T>> {
            crate::Object(Some(crate::rcmut::new(Array { data })))
        }

        pub fn placebos_usize(length: usize) -> crate::Object<Array<MaybeUninit<T>>> {
            let x = crate::array::placebos_box_usize::<T>(length);
            crate::rcmut::Array::<MaybeUninit<T>>::new(x)
        }
    }
    /// A reference counted smart pointer with unrestricted mutability.
    pub type RcMut<T> = Rc<UnsafeCell<T>>;

    /// Create a new RcMut for a value.
    pub fn new<T>(val: T) -> RcMut<T> {
        Rc::new(UnsafeCell::new(val))
    }
    /// Retrieve the inner Rc as a reference.
    pub unsafe fn from<T>(value: Box<T>) -> RcMut<T> {
        mem::transmute(Rc::new(*value))
    }

    pub unsafe fn from_rc<T: ?Sized>(value: Rc<T>) -> RcMut<T> {
        mem::transmute(value)
    }

    pub unsafe fn as_rc<T: ?Sized>(this: &RcMut<T>) -> &Rc<T> {
        mem::transmute(this)
    }
    pub unsafe fn to_rc<T: ?Sized>(this: RcMut<T>) -> Rc<T> {
        mem::transmute(this)
    }

    /// Retrieve the inner Rc as a mutable reference.
    pub unsafe fn as_rc_mut<T: ?Sized>(this: &mut RcMut<T>) -> &mut Rc<T> {
        mem::transmute(this)
    }

    /// Get a reference to the value.
    #[inline]
    pub unsafe fn borrow<T: ?Sized>(this: &RcMut<T>) -> &T {
        mem::transmute(this.get())
    }

    /// Get a mutable reference to the value.
    #[inline]
    pub unsafe fn borrow_mut<T: ?Sized>(this: &mut RcMut<T>) -> &mut T {
        mem::transmute(this.get())
    }

    /// A reference counted smart pointer with unrestricted mutability.
    pub struct ArcMut<T: ?Sized> {
        inner: Arc<UnsafeCell<T>>,
    }

    impl<T: ?Sized> Clone for ArcMut<T> {
        fn clone(&self) -> ArcMut<T> {
            ArcMut {
                inner: self.inner.clone(),
            }
        }
    }

    impl<T> ArcMut<T> {
        /// Create a new ArcMut for a value.
        pub fn new(val: T) -> ArcMut<T> {
            ArcMut {
                inner: Arc::new(UnsafeCell::new(val)),
            }
        }
    }

    impl<T: ?Sized> ArcMut<T> {
        /// Retrieve the inner Rc as a reference.
        pub unsafe fn as_arc(&self) -> &Arc<T> {
            mem::transmute(&self.inner)
        }

        /// Retrieve the inner Rc as a mutable reference.
        pub unsafe fn as_arc_mut(&mut self) -> &mut Arc<T> {
            mem::transmute(&mut self.inner)
        }

        /// Get a reference to the value.
        pub unsafe fn borrow(&self) -> &T {
            mem::transmute(self.inner.get())
        }

        /// Get a mutable reference to the value.
        pub unsafe fn borrow_mut(&mut self) -> &mut T {
            mem::transmute(self.inner.get())
        }
    }
}

/////////////////
// Method helpers
/////////////////

// A MaybePlacebo is a value that is either a placebo or a real value.
// It is a wrapper around a MaybeUninit<T> value, but knowing whether the value is a placebo or not.
// That way, when it is dropped, the underlying value is only dropped if it is not a placebo.
pub struct MaybePlacebo<T>(Option<T>);
impl<T: Clone> MaybePlacebo<T> {
    #[inline]
    pub fn read(&self) -> T {
        // safety: Dafny will guarantee we will never read a placebo value
        unsafe { self.0.clone().unwrap_unchecked() }
    }
}

impl<T> MaybePlacebo<T> {
    #[inline]
    pub fn new() -> Self {
        MaybePlacebo(None)
    }
    #[inline]
    pub fn from(v: T) -> Self {
        MaybePlacebo(Some(v))
    }
}

#[macro_export]
macro_rules! tuple_extract_index {
    ($x:expr, $i:expr) => {
        $x.$i
    };
}

// A macro that maps tuple (a, b, c...) to produce (MaybePlacebo::from(a), MaybePlacebo::from(b), MaybePlacebo::from(c))
// maybe_placebos_from!(expr, 0, 1, 2, 3)
// = let x = expr;
//   (MaybePlacebo::from(x.0), MaybePlacebo::from(x.1), MaybePlacebo::from(x.2), MaybePlacebo::from(x.3))
#[macro_export]
macro_rules! maybe_placebos_from {
    ($x:expr, $($i:tt), *) => {
        {
            let x = $x;
            (
                $( $crate::MaybePlacebo::from(x.$i), )*
            )
        }
    }
}

////////////////
// Coercion
////////////////

// To use this trait, one needs to clone the element before.
pub trait UpcastTo<U>: Clone {
    fn upcast_to(self) -> U;
}

#[macro_export]
macro_rules! UpcastTo {
    ($from:ty, $to:ty) => {
        impl $crate::UpcastTo<*mut $to> for & $from {
            fn upcast_to(self) -> *mut $to {
                self as *const $to as *mut $to
            }
        }
    };
}
use nightly_crimes::nightly_crimes;
nightly_crimes! {
    #![feature(unsize)]
    impl<From, To> UpcastTo<::std::rc::Rc<To>> for ::std::rc::Rc<From>
    where
        From: ?Sized + core::marker::Unsize<To>,
        To: ?Sized,
    {
        fn upcast_to(self) -> ::std::rc::Rc<To> {
        self as ::std::rc::Rc<To>
        }
    }
}

#[macro_export]
macro_rules! UpcastToRc {
    ($from:ty, $to:ty) => {
        impl $crate::UpcastTo<::std::rc::Rc<$to>> for ::std::rc::Rc<$from> {
            fn upcast_to(&self) -> ::std::rc::Rc<$to> {
                (*self) as *const $to as *mut $to
            }
        }
    };
}

// UpcastTo for pointers
impl<T: 'static> UpcastTo<*mut dyn Any> for *mut T {
    fn upcast_to(self) -> *mut dyn Any {
        self as *const dyn Any as *mut dyn Any
    }
}

impl <From, To> UpcastTo<Object<To>> for Object<From>
where
    To: ?Sized,
    Rc<From>: UpcastTo<Rc<To>>,
{
    fn upcast_to(self) -> Object<To> {
        Object(Some(unsafe { rcmut::from_rc(rcmut::to_rc(self.0.clone().unwrap()).upcast_to()) }))
    }
}

// UpcastTo for sets
impl<V, U> UpcastTo<Set<V>> for Set<U>
where
    V: DafnyTypeEq,
    U: DafnyTypeEq + UpcastTo<V>,
{
    fn upcast_to(self) -> Set<V> {
        // We need to upcast individual elements
        let mut new_set: HashSet<V> = HashSet::<V>::default();
        for value in self.data.iter() {
            new_set.insert(value.clone().upcast_to());
        }
        Set::from_hashset_owned(new_set)
    }
}

// UpcastTo for sequences
impl<V, U> UpcastTo<Sequence<V>> for Sequence<U>
where
    V: DafnyTypeEq,
    U: DafnyTypeEq + UpcastTo<V>,
{
    fn upcast_to(self) -> Sequence<V> {
        // We need to upcast individual elements
        let mut new_seq: Vec<V> = Vec::<V>::default();
        for value in self.to_array().iter() {
            new_seq.push(value.clone().upcast_to());
        }
        Sequence::from_array_owned(new_seq)
    }
}

// Upcast for multisets
impl<V, U> UpcastTo<Multiset<V>> for Multiset<U>
where
    V: DafnyTypeEq,
    U: DafnyTypeEq + UpcastTo<V>,
{
    fn upcast_to(self) -> Multiset<V> {
        // We need to upcast individual elements
        let mut new_multiset: HashMap<V, DafnyInt> = HashMap::<V, DafnyInt>::default();
        for (value, count) in self.data.into_iter() {
            new_multiset.insert(value.upcast_to(), count.clone());
        }
        Multiset::from_hashmap_owned(new_multiset)
    }
}

// Upcast for Maps
impl<K, U, V> UpcastTo<Map<K, V>> for Map<K, U>
where
    K: DafnyTypeEq,
    U: DafnyTypeEq + UpcastTo<V>,
    V: DafnyTypeEq,
{
    fn upcast_to(self) -> Map<K, V> {
        // We need to upcast individual elements
        let mut new_map: HashMap<K, V> = HashMap::<K, V>::default();
        for (key, value) in self.data.iter() {
            new_map.insert(key.clone(), value.clone().upcast_to());
        }
        Map::from_hashmap_owned(new_map)
    }
}
