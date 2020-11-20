#![no_std]
#![feature(min_const_generics,maybe_uninit_extra)]

use core::mem::{self, MaybeUninit};

#[derive(Clone,Debug,PartialEq)]
pub enum ToArrayError {
    TooShort(usize, usize),
    TooLong(usize)
}

pub trait ToArray<T> {
    /// Convert to an array with zero allocations
    fn take_array<const N: usize>(&mut self) -> Result<[T; N], ToArrayError>;
    fn to_array<const N: usize>(self) -> Result<[T; N], ToArrayError>;
}

impl<I, T: Sized> ToArray<T> for I where I: Iterator<Item=T> {
    fn take_array<const N: usize>(&mut self) -> Result<[T; N], ToArrayError> {
        let mut res: [MaybeUninit<T>; N] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        
        let mut error_index = None;
        
        for (i, el) in res.iter_mut().enumerate() {
            if let Some(x) = self.next() {
                *el = MaybeUninit::new(x);
            } else {
                error_index = Some(i);
                break;
            }
        }
        
        if let Some(i) = error_index {
            // drop initialized elements
            for el in &mut res[..i] {
                unsafe { el.assume_init_drop() };
            } 
            Err(ToArrayError::TooShort(i, N))
        } else {
            Ok(unsafe {
                mem::transmute_copy(&res)
            })
        }
    } 
    
    fn to_array<const N: usize>(mut self) -> Result<[T; N], ToArrayError> {
        let arr = self.take_array()?;
        match self.next() {
            Some(_) => Err(ToArrayError::TooLong(N)),
            None => Ok(arr)
        } 
    }
}

pub trait ToArrayDefault<T> {
    fn take_array_default<const N: usize>(&mut self) -> [T; N];
    fn to_array_default<const N: usize>(self) -> Result<[T; N], ToArrayError>;
}

impl<I, T: Sized + Default> ToArrayDefault<T> for I where I: Iterator<Item=T> {
    fn take_array_default<const N: usize>(&mut self) -> [T; N] {
        let mut res: [MaybeUninit<T>; N] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        
        for el in &mut res {
            *el = MaybeUninit::new(self.next().unwrap_or_else(|| Default::default()));
        }
        unsafe {
            mem::transmute_copy(&res)
        }
    } 
    
    fn to_array_default<const N: usize>(mut self) -> Result<[T; N], ToArrayError> {
        let arr = self.take_array_default();
        match self.next() {
            Some(_) => Err(ToArrayError::TooLong(N)),
            None => Ok(arr)
        } 
    }
}

pub trait ToArrayPad<T> {
    fn take_array_pad<const N: usize>(&mut self, pad: T) -> [T; N];
    fn to_array_pad<const N: usize>(self, pad: T) -> Result<[T; N], ToArrayError>;
}

impl<I, T: Sized + Clone> ToArrayPad<T> for I where I: Iterator<Item=T> {
    fn take_array_pad<const N: usize>(&mut self, pad: T) -> [T; N] {
        let mut res: [MaybeUninit<T>; N] = unsafe {
            MaybeUninit::uninit().assume_init()
        };
        
        for el in &mut res {
            *el = MaybeUninit::new(self.next().unwrap_or_else(|| pad.clone()));
        }
        unsafe {
            mem::transmute_copy(&res)
        }
    } 
    
    fn to_array_pad<const N: usize>(mut self, pad: T) -> Result<[T; N], ToArrayError> {
        let arr = self.take_array_pad(pad);
        match self.next() {
            Some(_) => Err(ToArrayError::TooLong(N)),
            None => Ok(arr)
        } 
    }
}


#[cfg(test)]
#[macro_use]
extern crate std;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_array() {
        assert_eq!((0..5).to_array(), Ok([0,1,2,3,4]));
        assert_eq!((0..5).to_array::<10>(), Err(ToArrayError::TooShort(5, 10)));
        assert_eq!((0..0).to_array::<10>(), Err(ToArrayError::TooShort(0, 10)));
        assert_eq!((0..5).to_array::<4>(), Err(ToArrayError::TooLong(4)));

        let mut iter = 0..10;
        assert_eq!(iter.take_array(), Ok([0,1,2,3,4]));
        assert_eq!(iter.take_array(), Ok([5,6,7]));
        assert_eq!(iter.take_array::<5>(), Err(ToArrayError::TooShort(2, 5)));
    }

    #[test]
    fn to_array_default() {
        assert_eq!((0..5).to_array_default(), Ok([0,1,2,3,4]));
        assert_eq!((0..5).to_array_default(), Ok([0,1,2,3,4,0,0]));
        assert_eq!((0..0).to_array_default(), Ok([0; 10]));
        assert_eq!((0..5).to_array_default::<4>(), Err(ToArrayError::TooLong(4)));

        let mut iter = 0..8;
        assert_eq!(iter.take_array_default(), [0,1,2,3,4]);
        assert_eq!(iter.take_array_default(), [5,6,7,0,0]);
        assert_eq!(iter.take_array_default(), [0,0,0,0,0]);
    }

    #[test]
    fn to_array_pad() {
        assert_eq!((0..5).to_array_pad(4), Ok([0,1,2,3,4]));
        assert_eq!((0..5).to_array_pad(4), Ok([0,1,2,3,4,4,4]));
        assert_eq!((0..0).to_array_pad(4), Ok([4; 10]));
        assert_eq!((0..5).to_array_pad::<4>(4), Err(ToArrayError::TooLong(4)));

        let mut iter = 0..8;
        assert_eq!(iter.take_array_pad(4), [0,1,2,3,4]);
        assert_eq!(iter.take_array_pad(4), [5,6,7,4,4]);
        assert_eq!(iter.take_array_pad(4), [4,4,4,4,4]);
    }

    #[test]
    fn array_of_vecs() {
        use std::vec::Vec;
        let v = vec![(1..5).collect::<Vec<i32>>(); 5];
        let arr = v.into_iter().to_array::<5>().unwrap();
        for x in &arr {
            assert_eq!(*x, vec![1i32,2,3,4])
        }
    }

    #[test]
    #[should_panic]
    fn array_of_vecs_fail() {
        use std::vec::Vec;
        let v = vec![(1..5).collect::<Vec<i32>>(); 5];
        v.into_iter().to_array::<6>().unwrap();
    }
}
