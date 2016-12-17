extern crate rs6502;

use std::io::{self, BufRead, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use rs6502::{Cpu, Disassembler};

#[derive(Debug)]
pub struct MemoryMonitor {
    pub enabled: bool,
    start_addr: usize,
    end_addr: usize,
}

pub struct VirtualMachine {
    pub cpu: Cpu,
    pub monitor: MemoryMonitor,
    code_offset: u16,
    clock_rate: Option<u32>,
    receiver: Receiver<String>,
    last_command: String,
}

impl VirtualMachine {
    pub fn new<CR>(cpu: Cpu, code_offset: u16, clock_rate: CR) -> VirtualMachine
        where CR: Into<Option<u32>>
    {
        let (tx, rx) = channel();

        thread::spawn(move || {
            std::io::stdout().write(b"hakka> ").unwrap();
            std::io::stdout().flush().unwrap();

            loop {
                let mut line = String::new();
                let stdin = io::stdin();
                stdin.lock().read_line(&mut line).expect("Could not read line");
                tx.send(line).unwrap();
            }
        });

        VirtualMachine {
            cpu: cpu,
            code_offset: code_offset,
            clock_rate: clock_rate.into(),
            receiver: rx,
            monitor: MemoryMonitor {
                enabled: false,
                start_addr: 0,
                end_addr: 0,
            },
            last_command: "".into(),
        }
    }

    pub fn cycle(&mut self) {
        if let Some(clock_rate) = self.clock_rate {
            let mut n = 0;
            while n < clock_rate {
                n += self.cpu.step().expect("SEGFAULT") as u32;
            }
        } else {
            self.cpu.step().unwrap();
        }
    }

    pub fn try_execute_command(&mut self) {
        if let Ok(input) = self.receiver.try_recv() {
            let mut input: String = input.trim().into();

            if input.len() == 0 {
                if self.monitor.enabled {
                    self.monitor.enabled = false;
                } else {
                    input = self.last_command.clone();
                }
            }

            if input == "list" {
                self.dump_disassembly();
            }

            if input.starts_with("monitor") {
                if self.monitor.enabled {
                    self.monitor.enabled = false;
                } else {
                    self.enable_memory_monitor(&input);
                }
            }

            if input.starts_with("memset") {
                let parts: Vec<&str> = input.split(" ").collect();
                if parts.len() < 3 {
                    println!("ERR: Requires 2 arguments. Example: memset 0x00 0x01 to store 0x01 \
                              in 0x00.");
                } else if parts.len() == 3 {
                    if let Ok(dst) = u16::from_str_radix(&parts[1].replace("0x", "")[..], 16) {
                        if let Ok(src) = u8::from_str_radix(&parts[2].replace("0x", "")[..], 16) {
                            self.cpu.memory[dst as usize] = src;
                        } else {
                            println!("ERR: Unable to parse source byte value");
                        }
                    } else {
                        println!("ERR: Unable to parse destination byte value");
                    }
                } else {
                    if let Ok(mut dst) = usize::from_str_radix(&parts[1].replace("0x", "")[..],
                                                               16) {
                        for p in &parts[2..] {
                            if let Ok(byte) = u8::from_str_radix(&p.replace("0x", "")[..], 16) {
                                self.cpu.memory[dst] = byte;
                                dst += 0x01;
                            } else {
                                println!("ERR: Unable to parse source byte value");
                            }
                        }
                    } else {
                        println!("ERR: Unable to parse destination byte value");
                    }
                }
            }

            if input.starts_with("memdmp") {
                // 1 argument assumes 1 memory "page"
                let parts: Vec<&str> = input.split(" ").collect();
                if parts.len() == 2 {
                    let page_number: usize =
                        parts[1].parse().expect("Unable to parse memory page number");

                    self.dump_memory_page(page_number);
                }

            }

            std::io::stdout().write(b"hakka> ").unwrap();
            std::io::stdout().flush().unwrap();

            // Don't assign a blank command as a last command
            if input.len() > 0 {
                self.last_command = input.clone();
            }
        }

    }

    fn enable_memory_monitor(&mut self, input: &str) {
        let parts: Vec<&str> = input.split(" ").collect();
        if parts.len() < 3 {
            println!("ERR: Requires 2 arguments. Example: monitor 0x00 0xFF");
        } else {
            let start = usize::from_str_radix(&parts[1].replace("0x", "")[..], 16).unwrap();
            let end = usize::from_str_radix(&parts[2].replace("0x", "")[..], 16).unwrap();

            self.monitor.start_addr = start;
            self.monitor.end_addr = end;
            self.monitor.enabled = true;

            println!("{:?}", self.monitor);
        }
    }

    fn dump_disassembly(&self) {
        println!("");
        println!("-- Disassembly --");

        let disassembler = Disassembler::with_offset(self.code_offset);
        let asm = disassembler.disassemble(self.cpu.get_code());
        print!("{}", asm);
    }

    pub fn dump_memory_page(&self, page: usize) {
        let mut addr = page * 0x100;
        println!("-- Memory dump --");
        for chunk in self.cpu.memory[page * 0x100..(page * 0x100) + 0x100].chunks(8) {
            print!("{:04X}: ", addr);
            for b in chunk {
                print!("{:02X} ", *b);
            }
            addr += 0x08;
            println!("");
        }
    }

    pub fn dump_memory(&self) {
        println!("-- Memory dump --");
        for chunk in self.cpu.memory[self.monitor.start_addr..self.monitor.end_addr + 0x01]
            .chunks(8) {
            for b in chunk {
                print!("{:02X} ", *b);
            }
            println!("");
        }
    }
}