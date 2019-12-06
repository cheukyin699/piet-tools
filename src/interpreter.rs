use crossterm::input::{input, InputEvent, KeyEvent};
use crossterm::screen::RawScreen;
use tui::layout::{Constraint, Direction, Layout};
use tui::backend::CrosstermBackend;
use tui::widgets::{Block, Borders, Paragraph, List, Text, Widget};
use tui::style::{Style, Modifier};
use tui::Terminal;

use std::io;

use crate::cmdconfig::CmdConfig;
use crate::cpu::{CPU, OpCode};

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

            if let Some(err) = &self.cpu.error {
                eprintln!("error: {}\n", err);
            }

            if let Some(output) = &self.cpu.output {
                print!("{}", output);
            }
        }
    }

    pub fn info(&self) {
        println!("{}", self.filename);
        print!("{}", self.cpu.get_info());
    }

    pub fn debug(&mut self) -> io::Result<()> {
        let stdout = io::stdout();
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;
        let _raw = RawScreen::into_raw_mode().unwrap();
        let input = input();
        let mut reader = input.read_async();

        terminal.clear()?;

        let mut running = true;
        let mut output_buffer: String = String::new();
        let mut error_buffer: String = String::new();
        // let mut codes: Vec<OpCode> = Vec::new();
        let info_title = format!("Info for {}", self.filename);

        while running {
            let stack = self.cpu.stack
                .iter()
                .map(|n| Text::raw(n.to_string()));
            let info = [
                Text::styled("DP: ", Style::default().modifier(Modifier::BOLD)),
                Text::raw(format!("{:?}\n", self.cpu.dp)),
                Text::styled("CC: ", Style::default().modifier(Modifier::BOLD)),
                Text::raw(format!("{:?}\n", self.cpu.cc)),
                Text::styled("PC: ", Style::default().modifier(Modifier::BOLD)),
                Text::raw(format!("{:?}\n", self.cpu.pc)),
                Text::styled("Last Command: ", Style::default().modifier(Modifier::BOLD)),
                Text::raw(format!("{:?}\n", self.cpu.last_cmd)),
            ];

            // Handle drawing things
            terminal.draw(|mut f| {
                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(50)].as_ref())
                    .split(f.size());

                let right_pane = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
                    .split(chunks[2]);

                let output_panes = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                    .split(right_pane[1]);

                // Code space
                // List::new(code)
                //     .block(Block::default().borders(Borders::ALL).title("Assembler"))
                //     .render(&mut f, chunks[0]);

                // Stack space
                List::new(stack)
                    .block(Block::default().borders(Borders::ALL).title("Stack"))
                    .render(&mut f, chunks[1]);

                // Info space
                Paragraph::new(info.iter())
                    .block(Block::default().borders(Borders::ALL).title(info_title.as_str()))
                    .render(&mut f, right_pane[0]);

                // Output space
                Paragraph::new(vec![Text::raw(output_buffer.as_str())].iter())
                    .block(Block::default().borders(Borders::ALL).title("STDOUT"))
                    .render(&mut f, output_panes[0]);

                // Error space
                Paragraph::new(vec![Text::raw(error_buffer.as_str())].iter())
                    .block(Block::default().borders(Borders::ALL).title("STDERR"))
                    .render(&mut f, output_panes[1]);
            })?;

            // Handle keypresses
            while let Some(event) = reader.next() {
                match event {
                    InputEvent::Keyboard(KeyEvent::Esc)
                    | InputEvent::Keyboard(KeyEvent::Char('q')) => running = false,
                    InputEvent::Keyboard(KeyEvent::Char('n')) => {
                        self.cpu.try_step();
                    },
                    _ => {}
                }
            }

            // Handle state updates
            if let Some(out) = &self.cpu.output {
                output_buffer += out.as_str();
                self.cpu.output = None;
            }
            if let Some(err) = &self.cpu.error {
                error_buffer += err.as_str();
                self.cpu.error = None;
            }
        }

        Ok(())
    }
}
