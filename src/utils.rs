use core::panic;

use crate::page::PageType;

pub struct ArrayIter<'a, T, const N: usize> {
    iter: std::slice::ChunksExact<'a, T>,
}

impl<'a, const N: usize, T> Iterator for ArrayIter<'a, T, N> {
    type Item = &'a [T; N];

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(|x| x.try_into().unwrap())
    }
}

pub trait MyCoolArrayStuff<T> {
    fn as_array_iter<'a, const N: usize>(&'a self) -> ArrayIter<'a, T, N>;
}

impl<T> MyCoolArrayStuff<T> for [T] {
    fn as_array_iter<'a, const N: usize>(&'a self) -> ArrayIter<'a, T, N> {
        ArrayIter {
            iter: self.chunks_exact(N),
        }
    }
}

struct ArrayIter2<'a, T, const N: usize> {
    underlying: &'a [T],
    index: usize,
}

impl<'a, const N: usize, T> Iterator for ArrayIter2<'a, T, N> {
    type Item = &'a [T; N];

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + N <= self.underlying.len() {
            let result = self.underlying[self.index..self.index + N]
                .try_into()
                .unwrap();
            self.index += N;
            Some(result)
        } else {
            None
        }
    }
}

trait MyCoolArrayStuff2<T> {
    fn as_array_iter2<'a, const N: usize>(&'a self) -> ArrayIter2<'a, T, N>;
}

impl<T> MyCoolArrayStuff2<T> for [T] {
    fn as_array_iter2<'a, const N: usize>(&'a self) -> ArrayIter2<'a, T, N> {
        ArrayIter2 {
            underlying: self,
            index: 0,
        }
    }
}

pub fn read_variant(bytes: &[u8]) -> (i64, usize) {
    let mut varint: i64 = 0;
    let mut bytes_read: usize = 0;
    for (i, byte) in bytes.iter().enumerate().take(9) {
        bytes_read += 1;
        if i == 8 {
            varint = (varint << 8) | *byte as i64;
            break;
        } else {
            varint = (varint << 7) | (*byte & 0b0111_1111) as i64;
            if *byte < 0b1000_0000 {
                break;
            }
        }
    }
    (varint, bytes_read)
}

pub fn get_page_type(t: u8) -> PageType {
    match t {
        2 => PageType::IndexInterior,
        5 => PageType::TableInterior,
        10 => PageType::IndexLeaf,
        13 => PageType::TableLeaf,
        _ => panic!("Something wrong"),
    }
}
