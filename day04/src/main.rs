#![feature(array_windows)]

fn main() {
    let mut answer1 = 0;
    let mut answer2 = 0;
    for n in 231832..=767346 {
        let password = n.to_string();
        if criteria(&password) {
            answer1 += 1;
        }
        if criteria2(&password) {
            answer2 += 1;
        }
    }
    println!("1: {}", answer1);
    println!("2: {}", answer2);
}

#[test]
fn test_criteria() {
    assert_eq!(criteria("111111"), true);
    assert_eq!(criteria("223450"), false);
    assert_eq!(criteria("123789"), false);
}

#[test]
fn test_criteria2() {
    assert_eq!(criteria2("112233"), true);
    assert_eq!(criteria2("123444"), false);
    assert_eq!(criteria2("111122"), true);
}

fn criteria(password: &str) -> bool {
    let mut repeat = false;
    for &[c0, c1] in password.as_bytes().array_windows::<2>() {
        if c0 == c1 {
            repeat = true;
        } else if c0 > c1 {
            return false;
        }
    }
    repeat
}

fn criteria2(password: &str) -> bool {
    let mut count = [0u8; 10];
    let b = password.as_bytes();
    count[(b[0] - b'0') as usize] += 1;
    for &[c0, c1] in b.array_windows::<2>() {
        if c0 > c1 {
            return false;
        }
        count[(c1 - b'0') as usize] += 1;
    }

    count.into_iter().any(|c| c == 2)
}
