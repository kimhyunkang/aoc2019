#![feature(map_try_insert)]

use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, Write},
    string::FromUtf8Error,
};

use intcode::VM;

pub fn utf8_error(e: FromUtf8Error) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, e)
}

pub fn write_console(data: &[isize]) -> io::Result<()> {
    let buf = read_ascii(data)?;
    let mut stdout = io::stdout();
    stdout.write_all(buf.as_bytes())
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

pub struct Graph {
    start: String,
    nodes: HashMap<String, Node>,
    edges: HashMap<String, HashMap<String, String>>,
}

impl Graph {
    pub fn scan(program: Vec<isize>) -> io::Result<Self> {
        let vm = VM::init(program);

        let mut start = String::new();
        let mut queue = VecDeque::new();
        let mut nodes = HashMap::new();
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

            if nodes.try_insert(title.clone(), node.clone()).is_err() {
                continue;
            }
            for (door, vm) in explore_node(vm, node.doors) {
                queue.push_back((Some((title.clone(), door)), vm));
            }
        }
        Ok(Self {
            start,
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
            println!("{}: {}, {:?}", name, p, self.nodes.get(name).unwrap().items);
        }
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
    desc: Vec<String>,
    doors: Vec<String>,
    items: Vec<String>,
}

impl Node {
    fn parse(output: &str) -> io::Result<Self> {
        let mut desc = Vec::new();
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
                        desc: vec![],
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
                0 => {
                    desc.push(line.to_string());
                }
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
            desc,
            doors,
            items,
        })
    }
}
