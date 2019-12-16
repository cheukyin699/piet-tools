use crate::blocks;
use crate::blocks::{Block, Blocks, Type};
use crate::cmdconfig::CmdConfig;
use crate::utils::Coord;

use std::io;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(i32)]
pub enum Direction {
    Right = 0,
    Down,
    Left,
    Up,
}

fn rotate_direction(d: Direction, times: i32) -> Direction {
    match ((d as i32) + times) % 4 {
        0 => Direction::Right,
        1 => Direction::Down,
        2 => Direction::Left,
        3 => Direction::Up,
        x => panic!("HOWTF did you manage to get {}???", x),
    }
}

fn switch_codel(c: Direction, times: i32) -> Direction {
    let times = if times < 0 { -times } else { times };
    if times % 2 == 0 {
        c
    } else {
        if c == Direction::Left {
            Direction::Right
        } else {
            Direction::Left
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum OpCode {
    NOP,
    PUSH,
    POP,
    ADD,
    SUB,
    MUL,
    DIV,
    MOD,
    NOT,
    GT,
    PTR,
    SWTCH,
    DUP,
    ROLL,
    INPN,
    INPC,
    OUTN,
    OUTC,
}

impl OpCode {
    const OPCODE_TABLE: [[OpCode; 3]; 6] = [
        [OpCode::NOP, OpCode::PUSH, OpCode::POP],
        [OpCode::ADD, OpCode::SUB, OpCode::MUL],
        [OpCode::DIV, OpCode::MOD, OpCode::NOT],
        [OpCode::GT, OpCode::PTR, OpCode::SWTCH],
        [OpCode::DUP, OpCode::ROLL, OpCode::INPN],
        [OpCode::INPC, OpCode::OUTN, OpCode::OUTC],
    ];

    pub fn typeof_exec(
        light0: blocks::Lightness,
        hue0: blocks::Hue,
        light1: blocks::Lightness,
        hue1: blocks::Hue,
    ) -> OpCode {
        let light_delta = ((light1 as i8) - (light0 as i8) + 3) % 3;
        let hue_delta = ((hue1 as i8) - (hue0 as i8) + 6) % 6;

        OpCode::OPCODE_TABLE[hue_delta as usize][light_delta as usize]
    }
}

pub struct CPU {
    codel_size: i32,
    code: Blocks,
    pub stack: Vec<i32>,
    pub dp: Direction,
    pub cc: Direction,
    pub pc: Coord,

    pub error: Option<String>,
    pub output: Option<String>,
    pub last_cmd: Option<OpCode>
}

impl CPU {
    pub fn from_config(cfg: &CmdConfig) -> CPU {
        match Blocks::from_file(cfg.src, cfg.size) {
            Ok(blocks) => CPU {
                codel_size: cfg.size,
                code: blocks,
                stack: vec![],
                dp: Direction::Right,
                cc: Direction::Left,
                pc: (0, 0),

                error: None,
                output: None,
                last_cmd: None,
            },
            Err(e) => panic!(e),
        }
    }

    pub fn get_info(&self) -> String {
        format!(
            "Total number of blocks: {}
Codel size: {}
# of codels: {}\n",
            self.code.len(),
            self.codel_size,
            self.code.count_codels()
        )
    }

    pub fn try_step(&mut self) -> bool {
        for i in 0..8 {
            if self.step() {
                break;
            }

            self.cc = switch_codel(self.cc, 1);
            if i % 2 == 1 {
                self.dp = rotate_direction(self.dp, 1);
            }

            if i == 7 {
                return false;
            }
        }
        true
    }

    fn step(&mut self) -> bool {
        let blk = self.code.find_block_from_index(&self.pc).unwrap();
        let edges = self.get_edges(blk);
        let (choose_x, choose_y) = self.choose_coord(edges);
        let new_coord = match self.dp {
            Direction::Right => (choose_x + self.codel_size, choose_y),
            Direction::Down => (choose_x, choose_y + self.codel_size),
            Direction::Left => (choose_x - self.codel_size, choose_y),
            Direction::Up => (choose_x, choose_y - self.codel_size),
        };
        let new_blk = match self.code.find_block_from_index(&new_coord) {
            Some(blk) => blk,
            None => return false,
        };

        self.error = None;
        self.output = None;
        let (vblk, vnewblk) = (blk.to_viewableblock(), new_blk.to_viewableblock());
        let success = self.execute_blk(vblk, vnewblk);
        if success {
            if self.pc == new_coord {
                return false;
            }
            self.pc = new_coord;
        }
        success
    }

    /// Executes the transition between blocks. Returns true if the block executes. The block does
    /// not execute if and only if the next block is black.
    ///
    /// This is basically a helper function that deals with the exceptional cases.
    fn execute_blk(&mut self, curr: blocks::ViewableBlock, next: blocks::ViewableBlock) -> bool {
        self.last_cmd = None;
        match next.t {
            Type::Black => false,
            Type::Color(l, h) => {
                if let Type::Color(l0, h0) = curr.t {
                    self.execute(curr, OpCode::typeof_exec(l0, h0, l, h));
                } else if let Type::Black = curr.t {
                    panic!(
                        "Your current block is {:?}, which is impossible",
                        curr.t
                    );
                }
                true
            }
            Type::White => true,
        }
    }

    fn execute(&mut self, curr: blocks::ViewableBlock, op: OpCode) -> Option<bool> {
        self.last_cmd = Some(op);
        match op {
            OpCode::NOP => {}
            OpCode::PUSH => {
                self.stack.push(curr.num as i32);
            }
            OpCode::POP => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                self.stack.pop();
            }
            OpCode::ADD => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (v1, v2) = (self.stack.pop()?, self.stack.pop()?);
                self.stack.push(v1 + v2);
            }
            OpCode::SUB => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (v1, v2) = (self.stack.pop()?, self.stack.pop()?);
                self.stack.push(v2 - v1);
            }
            OpCode::MUL => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (v1, v2) = (self.stack.pop()?, self.stack.pop()?);
                self.stack.push(v1 * v2);
            }
            OpCode::DIV => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (v1, v2) = (self.stack.pop()?, self.stack.pop()?);
                if v1 != 0 {
                    self.stack.push(v2 / v1);
                } else {
                    self.error = Some("Dividing by zero; skipping".to_string());
                    self.stack.push(v2);
                    self.stack.push(v1);
                }
            }
            OpCode::MOD => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (v1, v2) = (self.stack.pop()?, self.stack.pop()?);
                if v1 != 0 {
                    self.stack.push(v2 % v1);
                } else {
                    self.error = Some("Modular arithmetic with zero as base; skipping".to_string());
                    self.stack.push(v2);
                    self.stack.push(v1);
                }
            }
            OpCode::NOT => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let v = self.stack.pop()?;
                self.stack.push(if v == 0 { 1 } else { 0 });
            }
            OpCode::GT => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (v1, v2) = (self.stack.pop()?, self.stack.pop()?);
                self.stack.push(if v2 > v1 { 1 } else { 0 });
            }
            OpCode::PTR => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let v = self.stack.pop()?;
                self.dp = rotate_direction(self.dp, v);
            }
            OpCode::SWTCH => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let v = self.stack.pop()?;
                self.cc = switch_codel(self.cc, v);
            }
            OpCode::DUP => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let v = self.stack.pop()?;
                self.stack.push(v);
                self.stack.push(v);
            }
            OpCode::ROLL => {
                if self.stack.len() < 2 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let (num_rolls, n) = (self.stack.pop()?, self.stack.pop()?);
                if num_rolls < 0 || self.stack.len() < n as usize {
                    return None;
                }
                // Copied from https://github.com/tessi/rpiet/blob/master/src/command.rs#L267,
                // because I don't know what I am doing with this command
                let num_rolls = num_rolls % n;
                let mut substack: Vec<_> = self
                    .stack
                    .drain(self.stack.len() - (n as usize)..)
                    .collect();
                if num_rolls > 0 {
                    substack.rotate_right(num_rolls as usize);
                } else {
                    substack.rotate_left(-num_rolls as usize);
                }
                self.stack.append(&mut substack);
            }
            OpCode::INPN => {
                let mut line: String = "".to_string();
                let stdin = io::stdin();
                match stdin.read_line(&mut line) {
                    Ok(_) => self.stack.push(match line.trim().parse() {
                        Ok(num) => num,
                        Err(_) => {
                            self.error = Some(format!("Couldn't parse input '{}'", line));
                            return None;
                        }
                    }),
                    Err(_) => self.error = Some("Couldn't parse input".to_string()),
                }
            }
            OpCode::INPC => {
                let mut line: String = "".to_string();
                let stdin = io::stdin();
                match stdin.read_line(&mut line) {
                    Ok(_) => self.stack.push(line.as_bytes()[0] as i32),
                    Err(_) => self.error = Some("Couldn't parse input".to_string()),
                }
            }
            OpCode::OUTN => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let n = self.stack.pop()?;
                self.output = Some(n.to_string());
            }
            OpCode::OUTC => {
                if self.stack.len() < 1 {
                    self.error = Some("Not enough values to pop; skipping".to_string());
                    return None;
                }
                let n = self.stack.pop()?;
                self.output = Some((n as u8 as char).to_string());
            }
        }

        None
    }

    fn get_edges(&self, blk: &Block) -> Vec<Coord> {
        match self.dp {
            Direction::Right => {
                // Greatest x value fixed
                let (fixed_x, _): Coord = blk.coords.iter().max_by_key(|(x, _)| x).unwrap().clone();
                blk.coords
                    .iter()
                    .filter_map(|(x, y)| {
                        if fixed_x == *x {
                            Some((x.clone(), y.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Coord>>()
            }
            Direction::Down => {
                // Greatest y value fixed
                let (_, fixed_y): Coord = blk.coords.iter().max_by_key(|(_, y)| y).unwrap().clone();
                blk.coords
                    .iter()
                    .filter_map(|(x, y)| {
                        if fixed_y == *y {
                            Some((x.clone(), y.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Coord>>()
            }
            Direction::Left => {
                // Smallest x value fixed
                let (fixed_x, _): Coord = blk.coords.iter().min_by_key(|(x, _)| x).unwrap().clone();
                blk.coords
                    .iter()
                    .filter_map(|(x, y)| {
                        if fixed_x == *x {
                            Some((x.clone(), y.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Coord>>()
            }
            Direction::Up => {
                // Smallest y value fixed
                let (_, fixed_y): Coord = blk.coords.iter().min_by_key(|(_, y)| y).unwrap().clone();
                blk.coords
                    .iter()
                    .filter_map(|(x, y)| {
                        if fixed_y == *y {
                            Some((x.clone(), y.clone()))
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<Coord>>()
            }
        }
    }

    fn choose_coord(&self, edges: Vec<Coord>) -> Coord {
        let mut edges = edges;
        edges.sort_by_key(|(x, y)| match self.dp {
            Direction::Left | Direction::Right => *y,
            Direction::Up | Direction::Down => *x,
        });
        match self.dp {
            Direction::Up | Direction::Right => {
                if self.cc == Direction::Left {
                    edges.first().unwrap().clone()
                } else {
                    edges.last().unwrap().clone()
                }
            }
            Direction::Left | Direction::Down => {
                if self.cc == Direction::Left {
                    edges.last().unwrap().clone()
                } else {
                    edges.first().unwrap().clone()
                }
            }
        }
    }
}
