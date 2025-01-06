fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let modules: Vec<usize> = input.lines().map(|n| n.parse().unwrap()).collect();
    println!("1: {}", modules.iter().map(|&n| fuel(n)).sum::<usize>());
    println!("2: {}", modules.iter().map(|&n| rec_fuel(n)).sum::<usize>());
}

fn fuel(n: usize) -> usize {
    (n / 3).checked_sub(2).unwrap_or(0)
}

fn rec_fuel(n: usize) -> usize {
    let iter = core::iter::successors(Some(fuel(n)), |&n| if n > 0 { Some(fuel(n)) } else { None });
    iter.sum::<usize>()
}

#[test]
fn test1() {
    assert_eq!(fuel(12), 2);
    assert_eq!(fuel(14), 2);
    assert_eq!(fuel(1969), 654);
    assert_eq!(fuel(100756), 33583);
}

#[test]
fn test2() {
    assert_eq!(rec_fuel(12), 2);
    assert_eq!(rec_fuel(1969), 966);
    assert_eq!(rec_fuel(100756), 50346);
}
