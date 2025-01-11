#![feature(map_try_insert)]
#![feature(iter_intersperse)]

use std::{
    collections::{HashMap, VecDeque},
    io::{self, Write},
    string::FromUtf8Error,
};

use bitvec::{bitbox, boxed::BitBox, slice::BitSlice};
use intcode::VM;
use log::log_enabled;

pub static ITEM_EXCEPTION: &'static [&str] = &[
    "infinite loop",
    "giant electromagnet",
    "escape pod",
    "photons",
    "molten lava",
];

pub fn utf8_error(e: FromUtf8Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

pub fn write_console(data: &[isize]) -> io::Result<()> {
    let buf = read_ascii(data)?;
    let mut stdout = io::stdout();
    stdout.write_all(buf.as_bytes())
}

pub fn write_log(data: &[isize]) -> io::Result<()> {
    let buf = read_ascii(data)?;
    for line in buf.lines() {
        log::info!("{}", line);
    }
    Ok(())
}

pub fn read_ascii(data: &[isize]) -> io::Result<String> {
    let buf = data
        .iter()
        .map(|&b| {
            b.try_into().map_err(|_| {
                io::Error::new(io::ErrorKind::InvalidData, format!("Invalid byte {}", b))
            })
        })
        .collect::<Result<Vec<u8>, _>>()?;
    String::from_utf8(buf).map_err(utf8_error)
}

pub fn convert_ascii(data: &[u8]) -> Vec<isize> {
    data.iter().map(|&b| b as isize).collect::<Vec<_>>()
}

pub fn run_vm(vm: &mut VM, command: &str) -> io::Result<String> {
    log::info!("{:?}", command);
    vm.write_port(&convert_ascii(command.as_bytes()));
    vm.write_port(&[b'\n' as isize]);
    let exit = vm.run().is_ready();
    let output = read_ascii(&vm.read_all())?;
    if log_enabled!(log::Level::Info) {
        for line in output.lines() {
            log::info!("{}", line);
        }
    }
    if exit {
        log::info!("HALT");
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "HALT"))
    } else {
        Ok(output)
    }
}

pub fn run_vm_may_halt(vm: &mut VM, command: &str) -> io::Result<(String, bool)> {
    log::info!("{:?}", command);
    vm.write_port(&convert_ascii(command.as_bytes()));
    vm.write_port(&[b'\n' as isize]);
    let exit = vm.run().is_ready();
    let output = read_ascii(&vm.read_all())?;
    if log_enabled!(log::Level::Info) {
        for line in output.lines() {
            log::info!("{}", line);
        }
    }
    if exit {
        log::info!("HALT");
    }
    Ok((output, exit))
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Vertex<'r> {
    pub node: &'r str,
    pub items: BitBox,
}

#[derive(Debug, Clone, Copy)]
pub enum Path<'r> {
    Door(&'r str),
    Take(&'r str),
}

pub struct Graph {
    pub start: String,
    pub items: Vec<String>,
    pub nodes: HashMap<String, Vec<usize>>,
    pub edges: HashMap<String, HashMap<String, String>>,
}

impl Graph {
    pub fn scan(program: Vec<isize>) -> io::Result<Self> {
        let vm = VM::init(program);

        let mut start = String::new();
        let mut queue = VecDeque::new();
        let mut node_idx = HashMap::new();
        let mut edges = HashMap::new();

        queue.push_back((None, vm));

        while let Some((edge, vm)) = queue.pop_front() {
            let (vm, node) = read_node(vm)?;
            let title = node.title.clone();
            if let Some((prev, door)) = edge {
                edges
                    .entry(prev)
                    .or_insert_with(HashMap::new)
                    .insert(door, title.clone());
            } else {
                start = title.clone();
            }

            if node_idx.try_insert(title.clone(), node.clone()).is_err() {
                continue;
            }
            for (door, vm) in explore_node(vm, node.doors) {
                queue.push_back((Some((title.clone(), door)), vm));
            }
        }

        let mut items = Vec::new();

        let nodes = node_idx
            .into_iter()
            .map(|(title, node)| {
                let node_items = node
                    .items
                    .into_iter()
                    .map(|item| {
                        let idx = items.len();
                        items.push(item);
                        idx
                    })
                    .collect::<Vec<_>>();
                (title, node_items)
            })
            .collect::<HashMap<_, _>>();

        Ok(Self {
            start,
            items,
            nodes,
            edges,
        })
    }

    pub fn search(&self) {
        let mut path: HashMap<&String, String> = HashMap::new();
        let mut queue = VecDeque::new();
        queue.push_back(&self.start);
        path.insert(&self.start, String::new());

        while let Some(name) = queue.pop_front() {
            let p = path.get(name).unwrap().clone();
            if let Some(edges) = self.edges.get(name) {
                for (door, n1) in edges {
                    if !path.contains_key(n1) {
                        let mut p = p.clone();
                        p.push(door.chars().next().unwrap().to_ascii_uppercase());
                        path.insert(n1, p);
                        queue.push_back(n1);
                    }
                }
            }
        }

        for (name, p) in path {
            println!(
                "{}: {}, [{}]",
                name,
                p,
                self.nodes
                    .get(name)
                    .unwrap()
                    .iter()
                    .map(|&idx| self.items[idx].as_str())
                    .intersperse(",")
                    .collect::<String>()
            );
        }
    }

    pub fn collect_items<'r>(&'r self, target: &Vertex<'r>) -> Option<Vec<Path<'r>>> {
        let mut queue: VecDeque<Vertex<'r>> = VecDeque::new();
        let mut path: HashMap<Vertex<'r>, Vec<Path<'r>>> = HashMap::new();
        let i_empty = bitbox![0; self.items.len()];
        let (i0, take) = self.add_items(self.start.as_str(), &target.items, &i_empty);
        let v0 = Vertex {
            node: &self.start,
            items: i0,
        };
        path.insert(v0.clone(), take.into_iter().map(Path::Take).collect());
        queue.push_back(v0);

        while let Some(v0) = queue.pop_front() {
            let p0 = path.get(&v0).unwrap().clone();
            if &v0 == target {
                return Some(p0.clone());
            }

            if let Some(edges) = self.edges.get(v0.node) {
                for (door, n1) in edges {
                    let (i1, take) = self.add_items(n1.as_str(), &target.items, &v0.items);
                    let v1 = Vertex {
                        node: n1,
                        items: i1,
                    };
                    if path.get(&v1).is_none() {
                        let mut p1 = p0.clone();
                        p1.push(Path::Door(door));
                        p1.extend(take.into_iter().map(Path::Take));
                        path.insert(v1.clone(), p1);
                        queue.push_back(v1);
                    }
                }
            }
        }

        None
    }

    fn add_items<'r>(
        &'r self,
        node: &str,
        target: &BitSlice,
        collected: &BitSlice,
    ) -> (BitBox, Vec<&'r str>) {
        let mut collected = BitBox::from_bitslice(collected);
        let mut path = vec![];
        if let Some(items) = self.nodes.get(node) {
            for &item in items {
                if target[item] {
                    collected.set(item, true);
                    path.push(self.items[item].as_str());
                }
            }
        }
        (collected, path)
    }
}

fn read_node(mut vm: VM) -> io::Result<(VM, Node)> {
    if vm.run().is_ready() {
        return Err(io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Program finished",
        ));
    }
    let node = Node::parse(&read_ascii(&vm.read_all())?)?;
    Ok((vm, node))
}

fn explore_node(vm: VM, doors: Vec<String>) -> Vec<(String, VM)> {
    doors
        .into_iter()
        .map(|door| {
            let mut vm = vm.clone();
            vm.write_port(&convert_ascii(door.as_bytes()));
            vm.write_port(&[b'\n' as isize]);
            (door, vm)
        })
        .collect()
}

#[derive(Debug, Clone)]
struct Node {
    title: String,
    doors: Vec<String>,
    items: Vec<String>,
}

impl Node {
    fn parse(output: &str) -> io::Result<Self> {
        let mut doors = Vec::new();
        let mut items = Vec::new();
        let mut title = String::new();

        let mut phase = 0;
        for line in output.lines() {
            if line.trim().is_empty() || line.starts_with("Command?") {
                continue;
            }
            if line.starts_with("==") && line.ends_with("==") {
                title = line[3..line.len() - 3].to_string();
                if title == "Pressure-Sensitive Floor" {
                    return Ok(Node {
                        title,
                        doors: vec![],
                        items: vec![],
                    });
                }
                continue;
            }
            if line.starts_with("Doors here lead:") {
                phase = 1;
                continue;
            }
            if line.starts_with("Items here:") {
                phase = 2;
                continue;
            }
            match phase {
                0 => {}
                1 if line.starts_with("- ") => {
                    doors.push(line[2..].to_string());
                }
                2 if line.starts_with("- ") => {
                    items.push(line[2..].to_string());
                }
                _ => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("Invalid input {:?} at {:?}", line, title),
                    ));
                }
            }
        }

        Ok(Self {
            title,
            doors,
            items,
        })
    }
}
