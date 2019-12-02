use crate::blocks::{Type, Block, Blocks};
use crate::blocks;
use crate::cmdconfig::CmdConfig;
use crate::utils::Coord;

pub enum Direction {
    Right,
    Down,
    Left,
    Up
}

#[derive(Clone, Copy)]
enum OpCode {
    NOP, PUSH, POP,
    ADD, SUB, MUL,
    DIV, MOD, NOT,
    GT, PTR, SWTCH,
    DUP, ROLL, INPN,
    INPC, OUTN, OUTC
}

impl OpCode {
    const OPCODE_TABLE: [[OpCode; 3]; 6] = [
        [OpCode::NOP, OpCode::PUSH, OpCode::POP],
        [OpCode::ADD, OpCode::SUB, OpCode::MUL],
        [OpCode::DIV, OpCode::MOD, OpCode::NOT],
        [OpCode::GT, OpCode::PTR, OpCode::SWTCH],
        [OpCode::DUP, OpCode::ROLL, OpCode::INPN],
        [OpCode::INPC, OpCode::OUTN, OpCode::OUTC]
    ];

    pub fn typeof_exec(light0: blocks::Lightness, hue0: blocks::Hue,
        light1: blocks::Lightness, hue1: blocks::Hue) -> OpCode {
        let light_delta: usize = (((light1 as i8) - (light0 as i8)) % 3) as usize;
        let hue_delta: usize = (((hue1 as i8) - (hue0 as i8)) % 6) as usize;

        OpCode::OPCODE_TABLE[hue_delta][light_delta]
    }
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
        let blk = self.code.find_block_from_index(&self.current);
        if blk.t == Type::White {
            self.passthrough_white();
            return;
        }
        let edges = self.get_edges(blk);
        let (choose_x, choose_y) = self.choose_coord(&edges);
        let new_coord = match self.dp {
            Direction::Right => (*choose_x + 1, *choose_y),
            Direction::Down => (*choose_x, *choose_y - 1),
            Direction::Left => (*choose_x - 1, *choose_y),
            Direction::Up => (*choose_x, *choose_y - 1)
        };
        let new_blk = self.code.find_block_from_index(&new_coord);

        self.execute_blk(blk, new_blk);
    }

    fn passthrough_white(&self) {
    }

    /// Executes the transition between blocks. Returns true if the block executes. The block does
    /// not execute if and only if the next block is black.
    ///
    /// This is basically a helper function that deals with the exceptional cases.
    fn execute_blk(&self, curr_blk: &Block, next_blk: &Block) -> bool {
        match next_blk.t {
            Type::Black => false,
            Type::White => true,
            Type::Color(l, h) => {
                if let Type::Color(l0, h0) = curr_blk.t {
                    self.execute(curr_blk, next_blk, OpCode::typeof_exec(l0, h0, l, h));
                } else {
                    panic!("Your current block is {:?}, which is impossible", curr_blk.t);
                }
                true
            }
        }
    }

    fn execute(&self, curr_blk: &Block, next_blk: &Block, op: OpCode) {
    }

    fn get_edges(&self, blk: &Block) -> Vec<Coord> {
        blk.coords.iter().cloned().collect()
    }

    fn choose_coord(&'a self, edges: &'a Vec<Coord>) -> &'a Coord {
        // TODO: implement that table!
        edges.first().unwrap()
    }

    pub fn info(&self) {
        println!("Total number of blocks: {}", self.code.len());
        println!("Codel size: {}", self.codel_size);
    }
}
