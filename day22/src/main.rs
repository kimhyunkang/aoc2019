#![feature(mixed_integer_ops_unsigned_sub)]

use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let ops = Op::parse(&input);
    let g1: Group<10007> = Group::build(&ops);
    println!("1: {}", g1.get_idx(2019));

    let g2: Group<119315717514047> = Group::build(&ops).exp(101741582076661);
    println!("2: {}", g2.get_card(2020));
}

// ax + b (mod M)
#[derive(Clone, Copy)]
struct Group<const M: usize> {
    a: usize,
    b: usize,
}

impl<const M: usize> Group<M> {
    fn one() -> Self {
        Self { a: 1, b: 0 }
    }

    fn apply(self, op: Op) -> Self {
        match op {
            Op::NewStack => {
                // y0 = ax+b (mod M)
                // y1 = -y0-1 (mod M)
                //    = -ax-(b+1) (mod M)
                Group {
                    a: M - self.a,
                    b: M.checked_sub(self.b + 1)
                        .unwrap_or_else(|| 2 * M - self.b - 1),
                }
            }
            Op::Cut(n) => {
                // y0 = ax + b (mod M)
                // y1 = ax + b - n (mod M)
                let b1 = self
                    .b
                    .checked_sub_signed(n)
                    .unwrap_or_else(|| (self.b + M).checked_sub_signed(n).unwrap());
                Group {
                    a: self.a,
                    b: b1 % M,
                }
            }
            Op::Increment(n) => {
                // y0 = ax + b (mod M)
                // y1 = nax + nb (mod M)
                Group {
                    a: mod_mult::<M>(self.a, n),
                    b: mod_mult::<M>(self.b, n),
                }
            }
        }
    }

    fn exp(self, exponent: usize) -> Self {
        // y1 = ax + b
        // y2 = a(ax + b) + b
        //    = a^2x + ab + b
        // y3 = a(a^2x + ab + b)
        //    = a^3 x + a^2 b + a b + b
        // yn = a^n x + (a^n - 1) b / (a - 1)
        let an = mod_exp::<M>(self.a, exponent);
        let a1 = self.a.checked_sub(1).unwrap_or_else(|| self.a + M - 1);
        let a1_inv = mod_inv::<M>(a1).unwrap();
        let an1 = an.checked_sub(1).unwrap_or_else(|| an + M - 1);
        Group {
            a: an,
            b: mod_mult::<M>(mod_mult::<M>(an1, a1_inv), self.b),
        }
    }

    fn build(ops: &[Op]) -> Self {
        let mut g = Self::one();
        for &op in ops.iter() {
            g = g.apply(op);
        }
        g
    }

    fn get_idx(&self, n: usize) -> usize {
        (mod_mult::<M>(n, self.a) + self.b) % M
    }

    fn get_card(&self, idx: usize) -> usize {
        // y = ax + b (mod M)
        // x = (y-b) / a (mod M)
        let a_inv = mod_inv::<M>(self.a).unwrap();
        let yb = idx.checked_sub(self.b).unwrap_or_else(|| M + idx - self.b);
        mod_mult::<M>(a_inv, yb)
    }
}

fn mod_mult<const M: usize>(lhs: usize, rhs: usize) -> usize {
    if let Some(p) = lhs.checked_mul(rhs) {
        p % M
    } else if let Some(p) = (lhs % M).checked_mul(rhs % M) {
        p % M
    } else {
        let mut sum = 0;
        let mut a = lhs;
        let mut b = rhs;

        while b != 0 {
            if b % 2 == 1 {
                sum = (sum + a) % M;
            }
            a = (a + a) % M;
            b = b >> 1;
        }
        sum
    }
}

fn mod_exp<const M: usize>(base: usize, exponent: usize) -> usize {
    let mut res = 1;
    let mut b = base % M;
    let mut exp = exponent;

    while exp != 0 {
        if exp % 2 == 1 {
            res = mod_mult::<M>(res, b) % M;
        }
        exp = exp >> 1;
        b = mod_mult::<M>(b, b);
    }
    res
}

pub fn mod_inv<const M: usize>(a: usize) -> Option<usize> {
    let mut s = 1;
    let mut old_s = 0;
    let mut r = a;
    let mut old_r = M;
    let mut pos = false;

    while r != 0 {
        let q = old_r / r;
        let next_r = old_r % r;
        old_r = r;
        r = next_r;

        let next_s = q * s + old_s;
        old_s = s;
        s = next_s;

        pos = !pos;
    }

    if old_r == 1 {
        if pos { Some(old_s) } else { Some(M - old_s) }
    } else {
        None
    }
}

#[derive(Debug, Clone, Copy)]
enum Op {
    Cut(isize),
    Increment(usize),
    NewStack,
}

impl Op {
    fn parse(input: &str) -> Vec<Self> {
        input.lines().map(Self::parse_line).collect()
    }

    fn parse_line(line: &str) -> Self {
        static CUT: Lazy<Regex> = Lazy::new(|| Regex::new(r"cut (-?[0-9]+)").unwrap());
        static INCREMENT: Lazy<Regex> =
            Lazy::new(|| Regex::new(r"deal with increment ([0-9]+)").unwrap());
        if let Some(caps) = CUT.captures(line) {
            let (_, [s]) = caps.extract();
            Op::Cut(s.parse().unwrap())
        } else if let Some(caps) = INCREMENT.captures(line) {
            let (_, [s]) = caps.extract();
            Op::Increment(s.parse().unwrap())
        } else if line == "deal into new stack" {
            Op::NewStack
        } else {
            panic!("Invalid shuffle: {:?}", line)
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn exec<T: Copy>(op: &Op, cards: &mut [T]) {
        match op {
            &Op::NewStack => {
                cards.reverse();
            }
            &Op::Cut(n) => {
                let n0 = n.unsigned_abs();
                if n > 0 {
                    cards.rotate_left(n0);
                } else if n < 0 {
                    cards.rotate_right(n0);
                }
            }
            &Op::Increment(n) => {
                let space = cards.to_vec();
                for (i, x) in space.into_iter().enumerate() {
                    cards[(i * n) % cards.len()] = x;
                }
            }
        }
    }

    fn factory(n: usize) -> Vec<usize> {
        (0..n).collect()
    }

    fn shuffle_naive<T: Copy>(ops: &[Op], mut cards: Vec<T>) -> Vec<T> {
        for op in ops {
            exec(op, &mut cards);
        }
        cards
    }

    #[test]
    fn test_shuffle_naive() {
        let ops = Op::parse(include_str!("shuffle1.txt"));
        assert_eq!(shuffle_naive(&ops, factory(10)), vec![
            0, 3, 6, 9, 2, 5, 8, 1, 4, 7
        ]);

        let ops = Op::parse(include_str!("shuffle2.txt"));
        assert_eq!(shuffle_naive(&ops, factory(10)), vec![
            3, 0, 7, 4, 1, 8, 5, 2, 9, 6
        ]);

        let ops = Op::parse(include_str!("shuffle3.txt"));
        assert_eq!(shuffle_naive(&ops, factory(10)), vec![
            6, 3, 0, 7, 4, 1, 8, 5, 2, 9
        ]);

        let ops = Op::parse(include_str!("shuffle4.txt"));
        assert_eq!(shuffle_naive(&ops, factory(10)), vec![
            9, 2, 5, 8, 1, 4, 7, 0, 3, 6
        ]);
    }

    #[test]
    fn test_shuffle() {
        let ops = Op::parse(include_str!("shuffle4.txt"));
        let cards = shuffle_naive(&ops, factory(13));
        let g: Group<13> = Group::build(&ops);
        for (idx, &x) in cards.iter().enumerate() {
            assert_eq!(g.get_idx(x), idx);
            assert_eq!(g.get_card(idx), x);
        }
    }

    #[test]
    fn test_shuffle_multiple() {
        let ops = Op::parse(include_str!("shuffle_test.txt"));
        let mut cards = factory(31);
        for _ in 0..8 {
            cards = shuffle_naive(&ops, cards);
        }
        let g: Group<31> = Group::build(&ops).exp(8);

        for (idx, &x) in cards.iter().enumerate() {
            assert_eq!(g.get_idx(x), idx);
            assert_eq!(g.get_card(idx), x);
        }
    }

    #[test]
    fn test_new_stack() {
        let op = Op::NewStack;

        let cards = shuffle_naive(&[op], factory(11));
        let g: Group<11> = Group::build(&[op]);
        for (idx, &x) in cards.iter().enumerate() {
            assert_eq!(g.get_idx(x), idx);
        }
    }

    #[test]
    fn test_cut() {
        let op = Op::Cut(-4);

        let cards = shuffle_naive(&[op], factory(11));
        let g: Group<11> = Group::build(&[op]);
        for (idx, &x) in cards.iter().enumerate() {
            assert_eq!(g.get_idx(x), idx);
        }

        let op = Op::Cut(6);

        let cards = shuffle_naive(&[op], factory(11));
        let g: Group<11> = Group::build(&[op]);
        for (idx, &x) in cards.iter().enumerate() {
            assert_eq!(g.get_idx(x), idx);
        }
    }

    #[test]
    fn test_increment() {
        let op = Op::Increment(3);

        let cards = shuffle_naive(&[op], factory(11));
        let g: Group<11> = Group::build(&[op]);
        for (idx, &x) in cards.iter().enumerate() {
            assert_eq!(g.get_idx(x), idx);
        }
    }

    #[test]
    fn test_op() {
        let mut cards = factory(10);
        exec(&Op::NewStack, &mut cards);
        assert_eq!(cards, vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

        let mut cards = factory(10);
        exec(&Op::Increment(3), &mut cards);
        assert_eq!(cards, vec![0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);
    }
}
