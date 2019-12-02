use crate::blocks::Blocks;
use crate::cmdconfig::CmdConfig;
use crate::utils::Coord;

enum Direction {
    Right,
    Down,
    Left,
    Up
}

pub struct Interpreter {
    codel_size: usize,
    code: Blocks,
    stack: Vec<i32>,
    dp: Direction,
    cc: Direction,
    finished: bool,
    current: Coord,

    verbose: bool
}

impl <'a> Interpreter {
    pub fn from_config(cfg: &'a CmdConfig) -> Interpreter {
        match Blocks::from_file(cfg.src, cfg.size) {
            Ok(blocks) => Interpreter {
                codel_size: cfg.size,
                code: blocks,
                stack: vec![],
                dp: Direction::Right,
                cc: Direction::Left,
                finished: false,
                current: (0, 0),

                verbose: cfg.verbose
            },
            Err(e) => panic!(e)
        }
    }

    pub fn run(&self) {
        while !self.finished {
            self.step();
        }
    }

    fn step(&self) {
    }

    pub fn info(&self) {
        println!("Total number of blocks: {}", self.code.len());
        println!("Codel size: {}", self.codel_size);
    }
}
