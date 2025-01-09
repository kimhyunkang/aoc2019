#![feature(iter_intersperse)]

use intcode::{VM, parse_program};

fn main() {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    println!("1: {}", eval(program.clone(), include_str!("walk.txt")));
    println!("2: {}", eval(program, include_str!("run.txt")));
}

fn eval(program: Vec<isize>, script: &str) -> isize {
    let mut vm = VM::init(program);
    load_springscript(&mut vm, script);
    let output = vm.run_ready();
    for line in output.split(|&n| n == b'\n' as isize) {
        if let Ok(s) = try_convert(line) {
            log::info!("{}", s);
        } else {
            return line[0];
        }
    }
    panic!("Did not return score");
}

fn remove_comment(script: &str) -> String {
    script
        .lines()
        .filter(|&line| !line.trim().is_empty() && !line.starts_with("//"))
        .flat_map(|line| [line, "\n"])
        .collect()
}

fn load_springscript(vm: &mut VM, script: &str) {
    let buf = remove_comment(script)
        .as_bytes()
        .iter()
        .map(|&b| b as isize)
        .collect::<Vec<_>>();
    vm.write_port(&buf);
}

fn try_convert(output: &[isize]) -> Result<String, ()> {
    let buf = output
        .iter()
        .map(|&b| b.try_into())
        .collect::<Result<Vec<u8>, _>>()
        .map_err(|_| ())?;
    String::from_utf8(buf).map_err(|_| ())
}

#[cfg(test)]
mod test {
    use super::*;
    use once_cell::sync::Lazy;
    use regex::Regex;

    fn log_init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    enum Op {
        Not,
        And,
        Or,
    }

    struct SpringScript {
        op: Op,
        r0: char,
        r1: char,
    }

    #[derive(Default)]
    struct SpringVM {
        t: bool,
        j: bool,
    }

    impl SpringScript {
        fn parse_line(line: &str) -> Self {
            static PATTERN: Lazy<Regex> =
                Lazy::new(|| Regex::new(r"(AND|OR|NOT) ([A-Z]) ([A-Z])").unwrap());
            let (_, [op, r0, r1]) = PATTERN.captures(line).unwrap().extract();
            let r0 = r0.chars().next().unwrap();
            let r1 = r1.chars().next().unwrap();
            let op = match op {
                "AND" => Op::And,
                "OR" => Op::Or,
                "NOT" => Op::Not,
                _ => unreachable!(),
            };
            SpringScript { op, r0, r1 }
        }

        fn parse(input: &str) -> Vec<Self> {
            remove_comment(input)
                .lines()
                .filter(|&line| line != "RUN" && line != "WALK")
                .map(Self::parse_line)
                .collect()
        }
    }

    impl SpringVM {
        fn run(&mut self, inst: &SpringScript, forward: &[bool]) {
            let r0 = self.get(inst.r0, forward);
            let r1 = self.get(inst.r1, forward);
            let v = match inst.op {
                Op::And => r0 & r1,
                Op::Or => r0 | r1,
                Op::Not => !r0,
            };
            self.set(inst.r1, v);
        }

        fn get(&self, name: char, forward: &[bool]) -> bool {
            match name {
                'T' => self.t,
                'J' => self.j,
                'A'..='I' => forward
                    .get(((name as u8) - b'A') as usize)
                    .copied()
                    .unwrap_or(true),
                _ => unimplemented!(),
            }
        }

        fn set(&mut self, name: char, value: bool) {
            match name {
                'T' => self.t = value,
                'J' => self.j = value,
                _ => unimplemented!(),
            }
        }
    }

    fn run_step(script: &[SpringScript], surface: &[bool]) -> bool {
        let mut vm = SpringVM::default();
        for inst in script {
            vm.run(inst, &surface[1..]);
        }
        vm.j
    }

    fn show_surface(surface: &[bool]) -> String {
        surface.iter().map(|&b| if b { '#' } else { '.' }).collect()
    }

    fn parse_surface(surface: &str) -> Vec<bool> {
        surface.chars().map(|ch| ch == '#').collect()
    }

    fn run(script: &[SpringScript], surface: &[bool]) -> Result<(), usize> {
        let mut idx = 0;
        while idx < surface.len() {
            log::debug!("{:width$}@", "", width = idx);
            log::debug!("{}", show_surface(surface));
            if !surface[idx] {
                return Err(idx);
            }
            let jmp = run_step(script, &surface[idx..]);
            if jmp {
                idx += 4;
            } else {
                idx += 1;
            }
        }
        Ok(())
    }

    #[test]
    fn test1() {
        log_init();

        let script = SpringScript::parse(include_str!("run.txt"));
        let surface = parse_surface("#####.#.#...#####");

        run(&script, &surface).expect("Failed to run");
    }

    #[test]
    fn test2() {
        log_init();

        let script = SpringScript::parse(include_str!("run.txt"));
        let surface = parse_surface("#####...##.#.####");

        run(&script, &surface).expect("Failed to run");
    }
}
