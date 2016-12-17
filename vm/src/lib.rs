extern crate rs6502;

use std::io::{self, Write};
use std::sync::mpsc::Receiver;

use rs6502::{Assembler, Cpu, Disassembler};

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
}

impl VirtualMachine {
    pub fn new<CR>(cpu: Cpu,
                   code_offset: u16,
                   clock_rate: CR,
                   receiver: Receiver<String>)
                   -> VirtualMachine
        where CR: Into<Option<u32>>
    {
        VirtualMachine {
            cpu: cpu,
            code_offset: code_offset,
            clock_rate: clock_rate.into(),
            receiver: receiver,
            monitor: MemoryMonitor {
                enabled: false,
                start_addr: 0,
                end_addr: 0,
            },
        }
    }

    pub fn cycle(&mut self) {
        if let Some(clock_rate) = self.clock_rate {
            let mut n = 0;
            while n < clock_rate {
                n += self.cpu.step().unwrap() as u32;
            }
        } else {
            self.cpu.step().unwrap();
        }
    }

    pub fn dump_memory(&self) {
        println!("-- Memory dump --");
        for chunk in self.cpu.memory[self.monitor.start_addr..self.monitor.end_addr].chunks(8) {
            for b in chunk {
                print!("{:02X} ", *b);
            }
            println!("");
        }
    }

    pub fn try_execute_command(&mut self) {
        if let Ok(input) = self.receiver.try_recv() {
            let input = input.trim();

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
                    let dst = u16::from_str_radix(&parts[1].replace("0x", "")[..], 16).unwrap();
                    let src = u8::from_str_radix(&parts[2].replace("0x", "")[..], 16).unwrap();

                    self.cpu.memory[dst as usize] = src;
                } else {
                    let mut dst = usize::from_str_radix(&parts[1].replace("0x", "")[..], 16)
                        .unwrap();
                    for p in &parts[2..] {
                        let byte = u8::from_str_radix(&p.replace("0x", "")[..], 16).unwrap();
                        self.cpu.memory[dst] = byte;
                        dst += 0x01;
                    }
                }
            }

            std::io::stdout().write(b"hakka> ");
            std::io::stdout().flush();
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

        let mut disassembler = Disassembler::with_offset(self.code_offset);
        let asm = disassembler.disassemble(self.cpu.get_code());
        print!("{}", asm);
    }
}