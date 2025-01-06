fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let layers = decode(input.trim().as_bytes(), 6, 25);
    println!("1: {}", answer1(&layers));
    println!("2:");
    let pixels = render(&layers);
    for row in pixels.chunks_exact(25) {
        for &p in row {
            if p == b'1' { print!("#") } else { print!(" ") }
        }
        println!()
    }
}

fn decode(input: &[u8], h: usize, w: usize) -> Vec<&[u8]> {
    assert_eq!(input.len() % (h * w), 0);

    input.chunks_exact(h * w).collect()
}

fn answer1(layers: &[&[u8]]) -> usize {
    let count = layers
        .iter()
        .map(|&layer| count_pixels(layer))
        .min_by_key(|count| count[0])
        .unwrap();
    count[1] * count[2]
}

fn count_pixels(data: &[u8]) -> [usize; 3] {
    let mut count = [0; 3];
    for &p in data {
        if let b'0'..=b'2' = p {
            count[(p - b'0') as usize] += 1;
        } else {
            panic!("Invalid input")
        }
    }
    count
}

fn render(layers: &[&[u8]]) -> Vec<u8> {
    let mut pixels = Vec::with_capacity(layers[0].len());
    for pos in 0..layers[0].len() {
        let pixel = (0..layers.len())
            .map(|i| layers[i][pos])
            .find(|&c| c != b'2')
            .unwrap_or(b'2');
        pixels.push(pixel);
    }
    pixels
}
