use std::borrow::Borrow;

use crate::debugger_command::DebuggerCommand;
use crate::inferior::{Inferior, Status};
use rustyline::error::ReadlineError;
use rustyline::Editor;

use crate::dwarf_data::{DwarfData, Error as DwarfError};
use std::collections::HashMap;

pub struct Breakpoint {
    pub addr: usize,
    pub orig_byte: u8,
}

pub struct Debugger {
    target: String,
    history_path: String,
    readline: Editor<()>,
    inferior: Option<Inferior>,
    debug_data: DwarfData,
    breakpoints: HashMap<usize, Option<Breakpoint>>,
}

impl Debugger {
    /// Initializes the debugger.
    pub fn new(target: &str) -> Debugger {
        // TODO (milestone 3): initialize the DwarfData
        let debug_data = match DwarfData::from_file(target) {
            Ok(val) => val,
            Err(DwarfError::ErrorOpeningFile) => {
                println!("Could not open file {}", target);
                std::process::exit(1);
            }
            Err(DwarfError::DwarfFormatError(err)) => {
                println!("Could not load debugging symbols from {}: {:?}", target, err);
                std::process::exit(1);
            }
        };
        debug_data.print();

        let history_path = format!("{}/.deet_history", std::env::var("HOME").unwrap());
        let mut readline = Editor::<()>::new();
        // Attempt to load history from ~/.deet_history if it exists
        let _ = readline.load_history(&history_path);

        Debugger {
            target: target.to_string(),
            history_path,
            readline,
            inferior: None,
            debug_data: debug_data,
            breakpoints: HashMap::new(),
        }
    }

    pub fn run(&mut self) {
        loop {
            match self.get_next_command() {
                DebuggerCommand::Run(args) => {
                    if let Some(inferior) = &mut self.inferior {
                        inferior.kill().expect("inferior.kill wasn't running");
                    }
                    if let Some(inferior) = Inferior::new(&self.target, &args) {
                        // Create the inferior
                        self.inferior = Some(inferior);
                        // TODO (milestone 1): make the inferior run
                        // You may use self.inferior.as_mut().unwrap() to get a mutable reference
                        // to the Inferior object
                        match self.inferior.as_mut().unwrap().continue_exec(&mut self.breakpoints) {
                            Ok(status) => match status {
                                Status::Exited(exit_status_code) => {
                                    self.inferior = None;
                                    println!("Child exited (status {})", exit_status_code);
                                }
                                Status::Signaled(signal) => {
                                    self.inferior = None;
                                    println!("Chile exited (signal {})", signal);
                                }
                                Status::Stopped(signal, rip) => {
                                    println!("Child stopped (signal {})", signal);
                                    if let Some(line) = self.debug_data.get_line_from_addr(rip) {
                                        println!("Stopped at {} : {}", line.file, line.number);
                                    }
                                }
                            },
                            Err(err) => println!("Inferior can't be woken up and execute: {}", err),
                        }
                    } else {
                        println!("Error starting subprocess");
                    }
                }
                DebuggerCommand::Quit => {
                    if let Some(inferior) = &mut self.inferior {
                        inferior.kill().expect("inferior.kill wasn't running");
                    }
                    return;
                }
                DebuggerCommand::Continue => {
                    if let Some(inferior) = &self.inferior {
                        match self.inferior.as_mut().unwrap().continue_exec(&mut self.breakpoints) {
                            Ok(status) => match status {
                                Status::Exited(exit_status_code) => {
                                    self.inferior = None;
                                    println!("Child exited (status {})", exit_status_code);
                                }
                                Status::Signaled(signal) => {
                                    self.inferior = None;
                                    println!("Chile exited (signal {})", signal);
                                }
                                Status::Stopped(signal, rip) => {
                                    println!("Child stopped (signal {})", signal);
                                    if let Some(line) = self.debug_data.get_line_from_addr(rip) {
                                        println!("Stopped at {} :{}", line.file, line.number);
                                    }
                                }
                            },
                            Err(err) => println!("Inferior can't be woken up and execute: {}", err),
                        }
                    } else {
                        // if there is no inferior stopped, continue fails.
                        println!("There is no inferior stopped!");
                    }
                    
                }
                DebuggerCommand::Backtrace => {
                    if let Some(inferior) = &self.inferior {
                        match inferior.print_backtrace(&self.debug_data) {
                            Ok(_) => {},
                            Err(err) => {
                                println!("Err print_backtrace: {}", err);
                            },
                        }
                    }
                }
                DebuggerCommand::Breakpoint(bp_target) => {
                    let addr: usize;
                    if bp_target.starts_with("*") {
                        addr = self.parse_address(&bp_target[1..]).unwrap();
                    } else if let Ok(line_number) = bp_target.parse::<usize>() {
                        if let Some(address) = self.debug_data.get_addr_for_line(None, line_number) {
                            addr = address;
                        } else {
                            println!("line number can't find the corresponding address");
                            continue;
                        }
                    } else if let Some(address) = self.debug_data.get_addr_for_function(None, &bp_target) {
                        addr = address;
                    } else {
                        println!("{} can't be parsed to a breakpoint target", bp_target);
                        println!("Usage: b | break | breakpoint *address | line | func");
                        continue;
                    }
                    println!("Set breakpoint {} at {}", self.breakpoints.len(), addr);
                    
                    // self.breakpoint.push(addr);

                    if let Some(inferior) = &mut self.inferior {
                        match inferior.write_byte(addr, 0xcc) {
                            Ok(orig_byte) => {
                                self.breakpoints.insert(addr, Some(Breakpoint { addr: addr, orig_byte: orig_byte }));
                            }
                            Err(err) => {
                                println!("Debugger::new breakpoint write_byte: {}", err);
                            }
                        }
                    } else {
                        self.breakpoints.insert(addr, None);
                    }
                }
            }
        }
    }

    /// This function prompts the user to enter a command, and continues re-prompting until the user
    /// enters a valid command. It uses DebuggerCommand::from_tokens to do the command parsing.
    ///
    /// You don't need to read, understand, or modify this function.
    fn get_next_command(&mut self) -> DebuggerCommand {
        loop {
            // Print prompt and get next line of user input
            match self.readline.readline("(deet) ") {
                Err(ReadlineError::Interrupted) => {
                    // User pressed ctrl+c. We're going to ignore it
                    println!("Type \"quit\" to exit");
                }
                Err(ReadlineError::Eof) => {
                    // User pressed ctrl+d, which is the equivalent of "quit" for our purposes
                    return DebuggerCommand::Quit;
                }
                Err(err) => {
                    panic!("Unexpected I/O error: {:?}", err);
                }
                Ok(line) => {
                    if line.trim().len() == 0 {
                        continue;
                    }
                    self.readline.add_history_entry(line.as_str());
                    if let Err(err) = self.readline.save_history(&self.history_path) {
                        println!(
                            "Warning: failed to save history file at {}: {}",
                            self.history_path, err
                        );
                    }
                    let tokens: Vec<&str> = line.split_whitespace().collect();
                    if let Some(cmd) = DebuggerCommand::from_tokens(&tokens) {
                        return cmd;
                    } else {
                        println!("Unrecognized command.");
                    }
                }
            }
        }
    }

    fn parse_address(&self, addr: &str) -> Option<usize> {
        let addr_without_0x = if addr.to_lowercase().starts_with("0x") {
            &addr[2..]
        } else {
            &addr
        };
        usize::from_str_radix(addr_without_0x, 16).ok()
    }

}
