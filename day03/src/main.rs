use enum_stringify::EnumStringify;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let (wire0, wire1) = load_wires(&input);
    println!("1: {}", answer1(&wire0, &wire1).unwrap());
    println!("2: {}", answer2(&wire0, &wire1).unwrap());
}

#[test]
fn test1() {
    let (input0, input1) = load_wires(include_str!("test1.txt"));
    assert_eq!(answer1(&input0, &input1), Some(159));
    let (input0, input1) = load_wires(include_str!("test2.txt"));
    assert_eq!(answer1(&input0, &input1), Some(135));
}

#[test]
fn test2() {
    let (input0, input1) = load_wires(include_str!("test1.txt"));
    assert_eq!(answer2(&input0, &input1), Some(610));
    let (input0, input1) = load_wires(include_str!("test2.txt"));
    assert_eq!(answer2(&input0, &input1), Some(410));
}

fn answer1(wire0: &[Wire], wire1: &[Wire]) -> Option<usize> {
    let cross = wire0
        .iter()
        .flat_map(|w0| wire1.iter().filter_map(|w1| w0.cross(w1)))
        .filter(|&(x, y)| x != 0 && y != 0)
        .collect::<Vec<_>>();
    cross
        .into_iter()
        .map(|(x, y)| x.unsigned_abs() + y.unsigned_abs())
        .min()
}

fn answer2(wire0: &[Wire], wire1: &[Wire]) -> Option<usize> {
    let cross = wire0
        .iter()
        .enumerate()
        .flat_map(|(i, w0)| {
            wire1
                .iter()
                .enumerate()
                .filter_map(move |(j, w1)| match w0.cross(w1) {
                    Some((0, 0)) | None => None,
                    Some(c) => Some((i, j, c)),
                })
        })
        .collect::<Vec<_>>();
    cross
        .into_iter()
        .map(|(i, j, p)| {
            let len0 = wire0[..i].iter().map(|w| w.len()).sum::<usize>();
            let len1 = wire1[..j].iter().map(|w| w.len()).sum::<usize>();
            let piece0 = wire0[i].cut(p);
            let piece1 = wire1[j].cut(p);
            len0 + len1 + piece0 + piece1
        })
        .min()
}

enum Wire {
    H { x: (isize, isize), y: isize },
    V { x: isize, y: (isize, isize) },
}

impl Wire {
    fn run(input: &[(Dir, usize)]) -> Vec<Self> {
        let mut x: isize = 0;
        let mut y: isize = 0;
        let mut wires = Vec::with_capacity(input.len());

        for &(dir, len) in input {
            match dir {
                Dir::R => {
                    let x1 = x.checked_add_unsigned(len).unwrap();
                    wires.push(Self::H { x: (x, x1), y });
                    x = x1;
                }
                Dir::U => {
                    let y1 = y.checked_add_unsigned(len).unwrap();
                    wires.push(Self::V { x, y: (y, y1) });
                    y = y1;
                }
                Dir::L => {
                    let x1 = x.checked_sub_unsigned(len).unwrap();
                    wires.push(Self::H { x: (x, x1), y });
                    x = x1;
                }
                Dir::D => {
                    let y1 = y.checked_sub_unsigned(len).unwrap();
                    wires.push(Self::V { x, y: (y, y1) });
                    y = y1;
                }
            }
        }
        wires
    }

    fn in_range(xs: &(isize, isize), x: &isize) -> bool {
        let min_x = core::cmp::min(xs.0, xs.1);
        let max_x = core::cmp::max(xs.0, xs.1);
        (min_x..=max_x).contains(x)
    }

    fn cut(&self, (x, y): (isize, isize)) -> usize {
        match self {
            Self::H { x: (x0, _), .. } => x.abs_diff(*x0),
            Self::V { y: (y0, _), .. } => y.abs_diff(*y0),
        }
    }

    fn len(&self) -> usize {
        match self {
            Self::H { x, .. } => x.0.abs_diff(x.1),
            Self::V { y, .. } => y.0.abs_diff(y.1),
        }
    }

    fn cross(&self, other: &Self) -> Option<(isize, isize)> {
        match (self, other) {
            (Self::H { x: hx, y: hy }, Self::V { x: vx, y: vy }) => {
                if Self::in_range(hx, vx) && Self::in_range(vy, hy) {
                    Some((*vx, *hy))
                } else {
                    None
                }
            }
            (Self::V { x: vx, y: vy }, Self::H { x: hx, y: hy }) => {
                if Self::in_range(hx, vx) && Self::in_range(vy, hy) {
                    Some((*vx, *hy))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(EnumStringify, Clone, Copy)]
enum Dir {
    R,
    U,
    L,
    D,
}

fn parse_wire(input: &str) -> Vec<(Dir, usize)> {
    input
        .split(',')
        .map(|s| {
            let dir: Dir = s[..1].parse().unwrap();
            let n: usize = s[1..].parse().unwrap();
            (dir, n)
        })
        .collect()
}

fn parse(input: &str) -> (Vec<(Dir, usize)>, Vec<(Dir, usize)>) {
    let mut iter = input.lines().map(|line| parse_wire(line));
    let wire0 = iter.next().unwrap();
    let wire1 = iter.next().unwrap();
    (wire0, wire1)
}

fn load_wires(input: &str) -> (Vec<Wire>, Vec<Wire>) {
    let (input0, input1) = parse(input);
    (Wire::run(&input0), Wire::run(&input1))
}
