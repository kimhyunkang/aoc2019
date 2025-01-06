use core::fmt;

fn main() {
    let input = std::fs::read_to_string("input.txt").unwrap();
    let code: Vec<usize> = input
        .trim()
        .split(',')
        .map(|s| s.parse().unwrap())
        .collect();
    let mut code1 = code.clone();
    code1[1] = 12;
    code1[2] = 2;
    println!("1: {}", VM::init(code1).run());
    println!("2: {}", answer2(code, 19690720));
}

fn answer2(code: Vec<usize>, target: usize) -> usize {
    let mut vm2 = ExprVM::init(code);
    let (n, v) = vm2.run().eval().solve(target).next().unwrap();
    n * 100 + v
}

#[test]
fn test1() {
    assert_eq!(test_run(vec![1, 0, 0, 0, 99]), vec![2, 0, 0, 0, 99]);
    assert_eq!(test_run(vec![2, 3, 0, 3, 99]), vec![2, 3, 0, 6, 99]);
    assert_eq!(test_run(vec![2, 4, 4, 5, 99, 0]), vec![
        2, 4, 4, 5, 99, 9801
    ]);
    assert_eq!(test_run(vec![1, 1, 1, 4, 99, 5, 6, 0, 99]), vec![
        30, 1, 1, 4, 2, 5, 6, 0, 99
    ]);
}

struct VM {
    mem: Vec<usize>,
    pc: usize,
}

enum VMError {
    Halt,
}

impl VM {
    fn init(code: Vec<usize>) -> Self {
        VM { mem: code, pc: 0 }
    }

    fn step(&mut self) -> Result<(), VMError> {
        let op = self.mem[self.pc];
        if op == 99 {
            return Err(VMError::Halt);
        }
        let x = self.read(self.read(self.pc + 1));
        let y = self.read(self.read(self.pc + 2));
        let v = match op {
            1 => x + y,
            2 => x * y,
            _ => unimplemented!("opcode {} unimplemented", op),
        };
        self.write(self.read(self.pc + 3), v);
        self.pc += 4;
        Ok(())
    }

    fn run(&mut self) -> usize {
        while let Ok(()) = self.step() {
            ()
        }
        self.read(0)
    }

    fn read(&self, addr: usize) -> usize {
        self.mem.get(addr).copied().unwrap_or(0)
    }

    fn write(&mut self, addr: usize, val: usize) {
        if self.mem.len() <= addr {
            self.mem.resize(addr + 1, 0);
        }
        self.mem[addr] = val;
    }
}

#[derive(Clone)]
enum Expr {
    Const(usize),
    N,
    V,
    NVal,
    VVal,
    Add(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
}

// an + bv + c
struct Eq {
    a: usize,
    b: usize,
    c: usize,
}

impl Eq {
    fn compute(&self, n: usize, v: usize) -> usize {
        self.a * n + self.b * v + self.c
    }

    fn solve(&self, target: usize) -> impl Iterator<Item = (usize, usize)> {
        (0..=99)
            .flat_map(|n| (0..=99).map(move |v| (n, v)))
            .filter(move |&(n, v)| self.compute(n, v) == target)
    }
}

impl Expr {
    fn eval(&self) -> Eq {
        match self {
            Expr::Const(n) => Eq { a: 0, b: 0, c: *n },
            Expr::N => Eq { a: 1, b: 0, c: 0 },
            Expr::V => Eq { a: 0, b: 1, c: 0 },
            Expr::NVal => panic!("Cannot evaluate (*n)"),
            Expr::VVal => panic!("Cannot evaluate (*v)"),
            Expr::Add(e0, e1) => {
                let x = e0.eval();
                let y = e1.eval();
                Eq {
                    a: x.a + y.a,
                    b: x.b + y.b,
                    c: x.c + y.c,
                }
            }
            Expr::Mul(e0, e1) => {
                let v0 = e0.eval();
                let v1 = e1.eval();
                if v0.a == 0 && v0.b == 0 {
                    Eq {
                        a: v0.c * v1.a,
                        b: v0.c * v1.b,
                        c: v0.c * v1.c,
                    }
                } else if v1.a == 0 && v1.b == 0 {
                    Eq {
                        a: v1.c * v0.a,
                        b: v1.c * v0.b,
                        c: v1.c * v0.c,
                    }
                } else {
                    panic!("Cannot evaluate ({:?})*({:?})", v0, v1)
                }
            }
        }
    }
}

impl fmt::Debug for Eq {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut terms = Vec::new();
        if self.a != 0 {
            terms.push(format!("{}n", self.a));
        }
        if self.b != 0 {
            terms.push(format!("{}v", self.b));
        }
        terms.push(format!("{}", self.c));
        write!(f, "{}", terms.join("+"))
    }
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expr::Const(n) => write!(f, "{}", *n),
            Expr::N => write!(f, "n"),
            Expr::V => write!(f, "v"),
            Expr::NVal => write!(f, "(*n)"),
            Expr::VVal => write!(f, "(*v)"),
            Expr::Add(e0, e1) => write!(f, "({:?}+{:?})", e0, e1),
            Expr::Mul(e0, e1) => write!(f, "({:?}*{:?})", e0, e1),
        }
    }
}

struct ExprVM {
    mem: Vec<Expr>,
    pc: usize,
}

impl ExprVM {
    fn init(code: Vec<usize>) -> Self {
        let mut mem = code.into_iter().map(|n| Expr::Const(n)).collect::<Vec<_>>();
        mem[1] = Expr::N;
        mem[2] = Expr::V;
        ExprVM { mem, pc: 0 }
    }

    fn step(&mut self) -> Result<(), VMError> {
        let op = self.mem[self.pc].clone();
        if let Expr::Const(99) = op {
            return Err(VMError::Halt);
        }
        let x = self.read_ptr(self.pc + 1);
        let y = self.read_ptr(self.pc + 2);
        let v = match op {
            Expr::Const(1) => Expr::Add(Box::new(x), Box::new(y)),
            Expr::Const(2) => Expr::Mul(Box::new(x), Box::new(y)),
            _ => unimplemented!("opcode {:?} unimplemented", op),
        };
        self.write_ptr(self.pc + 3, v);
        self.pc += 4;
        Ok(())
    }

    fn run(&mut self) -> Expr {
        while let Ok(()) = self.step() {
            ()
        }
        self.read(0)
    }

    fn read_ptr(&self, addr: usize) -> Expr {
        match self.read(addr) {
            Expr::Const(ptr) => self.read(ptr),
            Expr::N => Expr::NVal,
            Expr::V => Expr::VVal,
            _ => panic!("Invalid pointer at {}: {:?}", addr, self.read(addr)),
        }
    }

    fn read(&self, addr: usize) -> Expr {
        self.mem.get(addr).cloned().unwrap_or(Expr::Const(0))
    }

    fn write_ptr(&mut self, addr: usize, val: Expr) {
        if let Expr::Const(ptr) = self.read(addr) {
            self.write(ptr, val)
        } else {
            panic!("Invalid pointer")
        }
    }

    fn write(&mut self, addr: usize, val: Expr) {
        if self.mem.len() <= addr {
            self.mem.resize(addr + 1, Expr::Const(0));
        }
        self.mem[addr] = val;
    }
}

#[cfg(test)]
fn test_run(code: Vec<usize>) -> Vec<usize> {
    let mut vm = VM::init(code);
    vm.run();
    vm.mem
}
