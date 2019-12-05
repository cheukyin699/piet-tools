use crate::cmdconfig::CmdConfig;
use crate::cpu::CPU;

pub struct Interpreter {
    cpu: CPU,
    filename: String,
}

impl Interpreter {
    pub fn from_config(cfg: &CmdConfig) -> Interpreter {
        Interpreter {
            cpu: CPU::from_config(cfg),
            filename: cfg.src.to_string(),
        }
    }

    pub fn run(&mut self) {
        let mut running = true;
        while running {
            running = self.cpu.try_step();
        }
    }
    pub fn info(&self) {
        println!("{}", self.filename);
        print!("{}", self.cpu.get_info());
    }
}
