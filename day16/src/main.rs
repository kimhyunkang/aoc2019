#![feature(portable_simd)]

use std::{
    ops::Mul,
    simd::{Simd, num::SimdInt},
};

use maligned::{A256, align_first_boxed};

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    println!("1: {}", Vector::parse(input.trim()).fft(100).prefix());
    println!("2: {}", answer2(input.trim()));
}

#[test]
fn test_multiply() {
    let m = Matrix::init(8);
    let x0: &[isize] = &[1, 2, 3, 4, 5, 6, 7, 8];
    let x1: &[isize] = &[4, 8, 2, 2, 6, 1, 5, 8];
    let v: Vector = x0.into();
    let v1 = &m * Vector::from(v);
    assert_eq!(v1, x1.into());
}

#[test]
fn test1() {
    assert_eq!(
        Vector::parse("80871224585914546619083218645595")
            .fft(100)
            .prefix(),
        "24176176"
    );

    assert_eq!(
        Vector::parse("19617804207202209144916044189917")
            .fft(100)
            .prefix(),
        "73745418"
    );

    assert_eq!(
        Vector::parse("69317163492948606335995924319873")
            .fft(100)
            .prefix(),
        "52432133"
    );
}

#[test]
fn test2() {
    assert_eq!(answer2("03036732577212944063491565474664"), "84462026");
    assert_eq!(answer2("02935109699940807407585447034323"), "78725270");
    assert_eq!(answer2("03081770884921959731165446850517"), "53553731");
}

fn answer2(input: &str) -> String {
    let offset: usize = input[..7].parse().unwrap();
    let digits = input
        .as_bytes()
        .iter()
        .map(|&c| c - b'0')
        .collect::<Vec<_>>();
    assert!(digits.len() * 5000 < offset);
    let len = digits.len() * 10000 - offset;
    let mut v = Vec::with_capacity(len);
    v.extend_from_slice(&digits[(offset % digits.len())..]);
    for _ in 0..(len / digits.len()) {
        v.extend_from_slice(&digits[..]);
    }
    assert_eq!(v.len(), len);
    for _ in 0..100 {
        let mut v1: Vec<u8> = Vec::with_capacity(len);
        let buf = v1.spare_capacity_mut();
        let mut acc = 0;
        for (b1, b0) in buf.iter_mut().zip(v).rev() {
            acc = (acc + b0) % 10;
            b1.write(acc);
        }
        unsafe {
            v1.set_len(len);
        }
        v = v1;
    }
    String::from_utf8(v[0..8].iter().map(|&b| b + b'0').collect::<Vec<_>>()).unwrap()
}

struct Matrix {
    m: Vec<Box<[isize]>>,
}

impl Matrix {
    fn init(n: usize) -> Self {
        Matrix {
            m: (1..=n)
                .map(|p| {
                    align_first_boxed::<isize, A256, _>(n, |i| {
                        let phase = (i + 1) % (4 * p);
                        match phase / p {
                            0 => 0,
                            1 => 1,
                            2 => 0,
                            3 => -1,
                            _ => unreachable!(),
                        }
                    })
                })
                .collect(),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
struct Vector {
    v: Box<[isize]>,
}

impl From<&[isize]> for Vector {
    fn from(xs: &[isize]) -> Self {
        Self {
            v: align_first_boxed::<isize, A256, _>(xs.len(), |i| xs[i]),
        }
    }
}

impl Vector {
    fn parse(input: &str) -> Self {
        let bytes = input.as_bytes();
        Self {
            v: align_first_boxed::<isize, A256, _>(bytes.len(), |i| (bytes[i] - b'0') as isize),
        }
    }

    fn fft(self, phase: usize) -> Self {
        let m = Matrix::init(self.v.len());
        let mut v = self;
        for _ in 0..phase {
            v = &m * v;
        }
        v
    }

    fn prefix(&self) -> String {
        String::from_utf8(
            self.v[..]
                .iter()
                .map(|&n| (n as u8) + b'0')
                .take(8)
                .collect(),
        )
        .unwrap()
    }
}

impl<'m> Mul<Vector> for &'m Matrix {
    type Output = Vector;

    fn mul(self, rhs: Vector) -> Self::Output {
        assert_eq!(self.m.len(), rhs.v.len());
        let (h, xchunks, xt) = rhs.v.as_simd::<4>();
        let mut xtail: Simd<isize, 4> = Simd::splat(0);
        xtail.as_mut_array()[..xt.len()].copy_from_slice(xt);

        assert_eq!(h.len(), 0);
        Vector {
            v: align_first_boxed::<isize, A256, _>(self.m.len(), |i| {
                let (h, mchunks, mt) = self.m[i].as_simd::<4>();
                assert_eq!(h.len(), 0);
                let mut sum: Simd<isize, 4> = Simd::splat(0);
                sum.as_mut_array()[..mt.len()].copy_from_slice(mt);
                sum *= xtail;

                for (&v, &m) in xchunks.iter().zip(mchunks) {
                    sum += v * m;
                }
                sum.reduce_sum().abs() % 10
            }),
        }
    }
}
