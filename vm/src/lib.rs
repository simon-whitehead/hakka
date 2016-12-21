extern crate rs6502;

use std::io::{self, BufRead, Write};
use std::sync::mpsc::{channel, Receiver};
use std::thread;

use rs6502::{CodeSegment, Cpu, Disassembler};

const HELPTEXT: &'static str = "

HAKKA
-----

Commands:

break | b: addr
       - Toggles a breakpoint at a specific address. If the
         program counter hits this address, execution
         stops. 
        
step | s
       - Executes the current instruction before breaking
         execution again.

continue | c
       - Resumes execution of code within the virtual machine.

list
       - Lists the code surrounding the current program
         counter.

memset | set: addr args [args, ...]
       - memset sets the value of memory directly.

memdmp | dmp: page || start end
       - memdmp dumps a single memory page, or a specified
         memory range from <start> to <end> (inclusive).

monitor | mon: start end
       - monitor dumps the memory between start and end (inclusive)
         every second. Press ENTER to stop the monitor.

source
       - Lists the code currently running in the virtual
         machine. A '>' symbol indicates the current 
         program counter.

registers | reg
       - Lists the CPU registers and their current values.

help
       - Lists this help text.
";

#[derive(Debug)]
pub struct MemoryMonitor {
    pub enabled: bool,
    start_addr: usize,
    end_addr: usize,
}

pub struct VirtualMachine {
    pub cpu: Cpu,
    pub monitor: MemoryMonitor,
    segments: Vec<CodeSegment>,
    clock_rate: Option<u32>,
    receiver: Receiver<String>,
    last_command: String,
    breakpoints: [u8; 64 * 1024],
    broken: bool,
    step: bool,
}

impl VirtualMachine {
    pub fn new<CR>(cpu: Cpu, clock_rate: CR) -> VirtualMachine
        where CR: Into<Option<u32>>
    {
        let (tx, rx) = channel();

        thread::spawn(move || {
            println!("Welcome to hakka. Type 'help' to get started.");
            println!("");

            std::io::stdout().write(b"hakka> ").unwrap();
            std::io::stdout().flush().unwrap();

            loop {
                let mut line = String::new();
                let stdin = io::stdin();
                let mut lock = stdin.lock();
                if let Ok(_) = lock.read_line(&mut line) {
                    tx.send(line).unwrap();
                } else {
                    break;
                }
            }
        });

        VirtualMachine {
            cpu: cpu,
            segments: Vec::new(),
            clock_rate: clock_rate.into(),
            receiver: rx,
            monitor: MemoryMonitor {
                enabled: false,
                start_addr: 0,
                end_addr: 0,
            },
            last_command: "".into(),
            breakpoints: [0; 64 * 1024],
            broken: false,
            step: false,
        }
    }

    pub fn load_code_segments(&mut self, segments: Vec<CodeSegment>) {
        if segments.len() == 0 {
            return;
        }
        self.segments = segments;
        for segment in &self.segments {
            self.cpu.load(&segment.code, segment.address);
        }

        self.cpu.registers.PC = self.segments[0].address;
    }

    /// Cycles the Virtual Machine CPU according to the clock rate
    pub fn cycle(&mut self) {
        if let Some(clock_rate) = self.clock_rate {
            let mut n = 0;
            while (n < clock_rate && !self.broken) || self.step {
                n += self.cpu.step().expect("SEGFAULT") as u32;
                if self.breakpoints[self.cpu.registers.PC as usize] > 0 {
                    self.broken = true;
                    println!("");
                    println!("BREAKPOINT hit at {:04X}", self.cpu.registers.PC);
                }
                // If we stepped, dump the local disassembly
                if self.step {
                    self.dump_local_disassembly();
                }
                self.step = false;
            }
        } else {
            self.cpu.step().expect("SEGFAULT");
            if self.step {
                self.dump_local_disassembly();
            }
            self.step = false;
            if self.breakpoints[self.cpu.registers.PC as usize] > 0 {
                self.broken = true;
                println!("");
                println!("BREAKPOINT hit at {:04X}", self.cpu.registers.PC);
            }
        }
    }

    pub fn try_execute_command(&mut self) {
        if let Ok(input) = self.receiver.try_recv() {
            let mut input: String = input.trim().into();

            if input.len() == 0 {
                if self.monitor.enabled {
                    self.monitor.enabled = false;
                }
            }

            if input == "r" || input == "repeat" {
                input = self.last_command.clone();
            }

            let parts = input.split(" ").collect::<Vec<_>>();

            if parts[0] == "help" {
                print!("{}", HELPTEXT);
            }

            if parts[0] == "source" {
                self.dump_disassembly();
            }

            if parts[0] == "list" {
                self.dump_local_disassembly();
            }

            if parts[0] == "monitor" || parts[0] == "mon" {
                if self.monitor.enabled {
                    self.monitor.enabled = false;
                } else {
                    self.enable_memory_monitor(&input);
                }
            }

            if parts[0] == "memset" || parts[0] == "set" {
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

            if parts[0] == "memdmp" || parts[0] == "dmp" {
                // 1 argument assumes 1 memory "page"
                if parts.len() == 2 {
                    if let Ok(page_number) = parts[1].parse() {

                        self.dump_memory_page(page_number);
                    } else {
                        println!("ERR: Unable to parse memory page");
                    }
                } else if parts.len() == 3 {
                    // A memory range instead
                    if let Ok(start) = u16::from_str_radix(&parts[1].replace("0x", "")[..], 16) {
                        if let Ok(end) = u16::from_str_radix(&parts[2].replace("0x", "")[..], 16) {
                            self.dump_memory_range(start, end);
                        } else {
                            println!("ERR: Unable to parse end address value");
                        }
                    } else {
                        println!("ERR: Unable to parse start address value");
                    }
                }
            }

            if parts[0] == "registers" || parts[0] == "reg" {
                self.dump_registers();
            }

            if parts[0] == "break" || parts[0] == "b" {
                // 1 argument assumes 1 memory "page"
                if parts.len() == 2 {
                    if let Ok(addr) = usize::from_str_radix(&parts[1][..], 16) {
                        if addr <= u16::max_value() as usize {
                            if self.breakpoints[addr] > 0 {
                                self.breakpoints[addr] = 0;
                            } else {
                                self.breakpoints[addr] = 1;
                            }
                        } else {
                            println!("ERR: Value outside addressable range.");
                        }
                    } else {
                        println!("ERR: Unable to parse breakpoint address");
                    }
                } else {
                    println!("ERR: Requires 1 argument");
                }
            }

            if parts[0] == "continue" || parts[0] == "c" {
                self.broken = false;
                println!("Execution resumed");
            }

            if parts[0] == "step" || parts[0] == "s" {
                self.step = true;
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
        }
    }

    fn dump_disassembly(&self) {
        println!("");
        println!("-- Disassembly --");

        for segment in &self.segments {
            println!(".ORG ${:04X}", segment.address);
            let disassembler = Disassembler::with_offset(segment.address);
            let pairs = disassembler.disassemble_with_addresses(&segment.code);
            let result = self.highlight_lines(self.cpu.registers.PC as usize,
                                 pairs,
                                 segment.address,
                                 false)
                .join("");
            print!("{}", result);
            println!("");
        }
    }

    fn dump_local_disassembly(&self) {
        println!("");
        println!("-- Disassembly --");

        let pc = self.cpu.registers.PC as usize;
        let local_segment = self.get_local_segment(pc);
        let disassembler = Disassembler::with_offset(local_segment.address);
        let pairs = disassembler.disassemble_with_addresses(&local_segment.code);
        let result = self.highlight_lines(pc, pairs, local_segment.address, true).join("");
        print!("{}", result);
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

    fn dump_memory_range(&self, start: u16, end: u16) {
        println!("-- Memory dump --");
        let start = start as usize;
        let end = end as usize;
        for chunk in self.cpu.memory[start..end + 0x01].chunks(8) {
            for b in chunk {
                print!("{:02X} ", *b);
            }
            println!("");
        }
    }

    fn dump_registers(&self) {
        println!("-- Registers --");
        println!("A: {} ({:04X})", self.cpu.registers.A, self.cpu.registers.A);
        println!("X: {} ({:04X})", self.cpu.registers.X, self.cpu.registers.X);
        println!("Y: {} ({:04X})", self.cpu.registers.Y, self.cpu.registers.Y);
        println!("PC: {} ({:04X})",
                 self.cpu.registers.PC,
                 self.cpu.registers.PC);
        println!("S: {} ({:04X})",
                 self.cpu.stack.pointer,
                 self.cpu.stack.pointer);
    }

    fn get_local_segment(&self, pc: usize) -> &CodeSegment {
        for segment in &self.segments {
            let addr = segment.address as usize;
            if pc >= addr && pc <= addr + segment.code.len() {
                return segment;
            }
        }

        &self.segments[0]
    }

    fn highlight_lines(&self,
                       pc: usize,
                       pairs: Vec<(String, u16)>,
                       segment_start: u16,
                       limit_results: bool)
                       -> Vec<String> {
        let mut result = Vec::new();

        let base = pc as isize - segment_start as isize;

        for pair in pairs {
            if limit_results {
                let start = if base > 0x0A { base - 0x0A } else { 0 };
                if (pair.1 as isize) < start || (pair.1 as isize) > base + 0x0A {
                    continue;
                }
            }
            let current_line = pc as u16 == segment_start + pair.1;
            let breakpoint = self.breakpoints[segment_start as usize + pair.1 as usize] > 0x00;

            if breakpoint && current_line {
                result.push(format!("> * {}", pair.0));
            } else if breakpoint && !current_line {
                result.push(format!("  * {}", pair.0));
            } else if !breakpoint && current_line {
                result.push(format!(">   {}", pair.0));
            } else {
                result.push(format!("    {}", pair.0));
            }
        }

        result
    }
}