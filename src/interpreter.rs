use crate::blocks::{Type, Block, Blocks};
use crate::blocks;
use crate::cmdconfig::CmdConfig;
use crate::utils::Coord;
use std::io;

#[derive(PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum Direction {
    Right = 0,
    Down,
    Left,
    Up
}

fn rotate_direction(d: Direction, times: i32) -> Direction {
    match ((d as i32) + times) % 4 {
        0 => Direction::Right,
        1 => Direction::Down,
        2 => Direction::Left,
        3 => Direction::Up,
        x => panic!("HOWTF did you manage to get {}???", x)
    }
}

fn switch_codel(c: Direction, times: i32) -> Direction {
    let times = if times < 0 {-times} else {times};
    if times % 2 == 0 {
        c
    } else {
        if c == Direction::Left {Direction::Right} else {Direction::Left}
    }
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
    stack: Vec<i32>,        // TODO: change to use a mutable reference with lifetimes
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

    pub fn run(&mut self) {
        while !self.finished {
            self.step();
        }
    }

    fn step(&mut self) {
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

        Interpreter::execute_blk(&mut self.stack, &mut self.dp, &mut self.cc, self.verbose, blk, new_blk);
    }

    fn passthrough_white(&self) {
    }

    /// Executes the transition between blocks. Returns true if the block executes. The block does
    /// not execute if and only if the next block is black.
    ///
    /// This is basically a helper function that deals with the exceptional cases.
    fn execute_blk(stack: &mut Vec<i32>, dp: &mut Direction, cc: &mut Direction, verbose: bool, curr_blk: &Block, next_blk: &Block) -> bool {
        match next_blk.t {
            Type::Black => false,
            Type::White => true,
            Type::Color(l, h) => {
                if let Type::Color(l0, h0) = curr_blk.t {
                    Interpreter::execute(stack, dp, cc, verbose, curr_blk, next_blk, OpCode::typeof_exec(l0, h0, l, h));
                } else {
                    panic!("Your current block is {:?}, which is impossible", curr_blk.t);
                }
                true
            }
        }
    }

    fn execute(stack: &mut Vec<i32>, dp: &mut Direction, cc: &mut Direction, verbose: bool, curr_blk: &Block, next_blk: &Block, op: OpCode) -> Option<bool> {
        match op {
            OpCode::NOP => {},
            OpCode::PUSH => {
                stack.push(curr_blk.coords.len() as i32);
                if verbose {
                    eprintln!("PUSH {:x}", curr_blk.coords.len());
                }
            },
            OpCode::POP => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                stack.pop();
                if verbose {
                    eprintln!("POP");
                }
            },
            OpCode::ADD => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (v1, v2) = (stack.pop()?, stack.pop()?);
                stack.push(v1 + v2);
                if verbose {
                    eprintln!("ADD {:x}, {:x}", v1, v2);
                }
            },
            OpCode::SUB => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (v1, v2) = (stack.pop()?, stack.pop()?);
                stack.push(v2 - v1);
                if verbose {
                    eprintln!("SUB {:x}, {:x}", v2, v1);
                }
            },
            OpCode::MUL => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (v1, v2) = (stack.pop()?, stack.pop()?);
                stack.push(v1 * v2);
                if verbose {
                    eprintln!("MUL {:x}, {:x}", v1, v2);
                }
            },
            OpCode::DIV => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (v1, v2) = (stack.pop()?, stack.pop()?);
                if v1 != 0 {
                    stack.push(v2 / v1);
                    if verbose {
                        eprintln!("DIV {:x}, {:x}", v2, v1);
                    }
                } else {
                    eprintln!("Dividing by zero; skipping");
                    stack.push(v2);
                    stack.push(v1);
                }
            },
            OpCode::MOD => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (v1, v2) = (stack.pop()?, stack.pop()?);
                if v1 != 0 {
                    stack.push(v2 % v1);
                    if verbose {
                        eprintln!("MOD {:x}, {:x}", v2, v1);
                    }
                } else {
                    eprintln!("Modular arithmetic with zero as base; skipping");
                    stack.push(v2);
                    stack.push(v1);
                }
            },
            OpCode::NOT => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let v = stack.pop()?;
                stack.push(if v == 0 {1} else {0});
                if verbose {
                    eprintln!("NOT {:x}", v);
                }
            },
            OpCode::GT => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (v1, v2) = (stack.pop()?, stack.pop()?);
                stack.push(if v2 > v1 {1} else {0});
                if verbose {
                    eprintln!("GT {:x}, {:x}", v2, v1);
                }
            },
            OpCode::PTR => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let v = stack.pop()?;
                *dp = rotate_direction(dp.clone(), v);
                if verbose {
                    eprintln!("PTR {:x}", v);
                }
            },
            OpCode::SWTCH => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let v = stack.pop()?;
                *cc = switch_codel(cc.clone(), v);
                if verbose {
                    eprintln!("SWTCH {:x}", v);
                }
            },
            OpCode::DUP => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let v = stack.pop()?;
                stack.push(v);
                stack.push(v);
                if verbose {
                    eprintln!("DUP {:x}", v);
                }
            },
            OpCode::ROLL => {
                if stack.len() < 2 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let (num_rolls, n) = (stack.pop()?, stack.pop()?);
                if num_rolls < 0 || stack.len() < n as usize {
                    return None;
                }
                // Copied from https://github.com/tessi/rpiet/blob/master/src/command.rs#L267,
                // because I don't know what I am doing with this command
                let num_rolls = num_rolls % n;
                let mut substack: Vec<_> = stack.drain(stack.len() - (n as usize)..).collect();
                if num_rolls > 0 {
                    substack.rotate_right(num_rolls as usize);
                } else {
                    substack.rotate_left(-num_rolls as usize);
                }
                stack.append(&mut substack);
                if verbose {
                    eprintln!("ROLL num={}, n={}", num_rolls, n);
                }
            },
            OpCode::INPN => {
                let mut line: String = "".to_string();
                let stdin = io::stdin();
                match stdin.read_line(&mut line) {
                    Ok(_) => stack.push(match line.parse() {
                        Ok(num) => {
                            if verbose {
                                eprintln!("INPN {}", line);
                            }
                            num
                        },
                        Err(_) => {
                            eprintln!("Couldn't parse input '{}'", line);
                            return None
                        }
                    }),
                    Err(_) => eprintln!("Couldn't parse input")
                }
            },
            OpCode::INPC => {
                let mut line: String = "".to_string();
                let stdin = io::stdin();
                match stdin.read_line(&mut line) {
                    Ok(_) => {
                        if verbose {
                            eprintln!("INPC {}", line);
                        }
                        stack.push(line.as_bytes()[0] as i32)
                    },
                    Err(_) => eprintln!("Couldn't parse input")
                }
            },
            OpCode::OUTN => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let n = stack.pop()?;
                print!("{}", n);
                if verbose {
                    eprintln!("OUTN {}", n);
                }
            },
            OpCode::OUTC => {
                if stack.len() < 1 {
                    eprintln!("Not enough values to pop; skipping");
                    return None;
                }
                let n = stack.pop()?;
                print!("{}", n as u8 as char);
                if verbose {
                    eprintln!("OUTN {}", n);
                }
            },
        }

        None
    }

    fn get_edges(&self, blk: &Block) -> Vec<Coord> {
        // TODO: implement getting the edge codels
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
