use crate::blocks::{Block, blocks_from_file};
use crate::cmdconfig::CmdConfig;

enum Direction {
    Right,
    Down,
    Left,
    Up
}

pub struct Interpreter {
    codel_size: usize,
    code: Vec<Block>,
    stack: Vec<i32>,
    dp: Direction,
    cc: Direction,

    verbose: bool
}

impl <'a> Interpreter {
    pub fn from_config(cfg: &'a CmdConfig) -> Interpreter {
        match blocks_from_file(cfg.src, cfg.size) {
            Ok(blocks) => Interpreter {
                codel_size: cfg.size,
                code: blocks,
                stack: vec![],
                dp: Direction::Right,
                cc: Direction::Left,

                verbose: cfg.verbose
            },
            Err(e) => panic!(e)
        }
    }

    pub fn run(&self) {
    }

    pub fn info(&self) {
        if self.verbose {
            for (i, b) in self.code.iter().enumerate() {
                println!("===Block #{}===", i);
                println!("{:#?}", b);
            }
        }
        println!("Total number of blocks: {}", self.code.len());
        println!("Codel size: {}", self.codel_size);
    }
}
