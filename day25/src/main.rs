use core::ascii;
use std::io;

use bitvec::{bitbox, boxed::BitBox, order::Lsb0, slice::BitSlice};
use day25::*;
use intcode::{VM, parse_program};

static CHECKPOINT: &'static str = "Security Checkpoint";
static PRESSURE_PLATE: &'static str = "Pressure-Sensitive Floor";

fn main() -> io::Result<()> {
    env_logger::init();

    let input = std::fs::read_to_string("input.txt").unwrap();
    let program = parse_program(&input);
    let graph = Graph::scan(program.clone())?;
    let target_items: BitBox = graph
        .items
        .iter()
        .map(|item| !ITEM_EXCEPTION.contains(&item.as_str()))
        .collect();
    let mut vm = VM::init(program);

    // Collect every items that can be picked up, and move right next to the pressure plate
    collect_and_checkpoint(&mut vm, &graph, target_items.clone())?;

    let item_ids: Vec<usize> = target_items.iter_ones().collect();
    let mut candidates: Vec<usize> = (0..(1 << item_ids.len())).collect();
    candidates.sort_by_key(|n| n.count_ones());

    for n in candidates {
        let mut items = bitbox![0; target_items.len()];
        for (i, &item_id) in item_ids.iter().enumerate() {
            if (n & (1 << i)) != 0 {
                items.set(item_id, true);
            }
        }
        let mut vm = vm.clone();
        if let Some(password) = check_weight(&mut vm, &graph, &items)? {
            println!("{}", password);
            return Ok(());
        }
    }

    println!("Not found");
    Ok(())
}

fn collect_and_checkpoint(vm: &mut VM, graph: &Graph, target_items: BitBox) -> io::Result<()> {
    let target = Vertex {
        node: &CHECKPOINT,
        items: target_items,
    };
    let path = graph.collect_items(&target).unwrap();
    for p in path {
        match p {
            Path::Door(door) => {
                run_vm(vm, door)?;
            }
            Path::Take(item) => {
                let command = format!("take {}", item);
                run_vm(vm, &command)?;
            }
        }
    }
    Ok(())
}

fn check_weight(vm: &mut VM, graph: &Graph, items: &BitSlice) -> io::Result<Option<String>> {
    for idx in items.iter_ones() {
        let item = &graph.items[idx];
        run_vm(vm, &format!("drop {}", item))?;
    }

    let (door, _room) = graph
        .edges
        .get(CHECKPOINT)
        .unwrap()
        .iter()
        .find(|(_door, room)| room.as_str() == PRESSURE_PLATE)
        .unwrap();
    let (ascii_output, exit) = run_vm_may_halt(vm, door)?;

    let analysis = ascii_output
        .lines()
        .find(|&line| line.starts_with("A loud, robotic voice says"))
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("analysis not found: {}", ascii_output),
            )
        })?;
    let voice = analysis.split('"').nth(1).unwrap();
    if !voice.starts_with("Analysis complete") {
        return Ok(None);
    }

    let password = ascii_output
        .lines()
        .find(|&line| line.starts_with("\"Oh, hello"))
        .map(|s| s.to_string());
    Ok(password)
}
