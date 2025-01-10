use once_cell::sync::Lazy;
use regex::Regex;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let ops = Op::parse(&input);
    let shuffled = shuffle_naive(&ops, factory(10007));
    println!("1: {}", shuffled[2019]);
}

#[derive(Debug)]
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

    fn exec<T: Copy>(&self, cards: &mut [T]) {
        match self {
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
}

fn factory(n: usize) -> Vec<usize> {
    (0..n).collect()
}

fn shuffle_naive<T: Copy>(ops: &[Op], mut cards: Vec<T>) -> Vec<T> {
    for op in ops {
        op.exec(&mut cards);
    }
    cards
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shuffle() {
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
    fn test_op() {
        let mut cards = factory(10);
        Op::NewStack.exec(&mut cards);
        assert_eq!(cards, vec![9, 8, 7, 6, 5, 4, 3, 2, 1, 0]);

        let mut cards = factory(10);
        Op::Increment(3).exec(&mut cards);
        assert_eq!(cards, vec![0, 7, 4, 1, 8, 5, 2, 9, 6, 3]);
    }
}
