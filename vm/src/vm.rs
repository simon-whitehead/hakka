
use rs6502::{CodeSegment, Cpu, Disassembler};
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;

use console::Console;

use std::io::Write;

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

pub struct VirtualMachine<'a> {
    pub cpu: Cpu,
    pub monitor: MemoryMonitor,
    pub console: Console<'a>,
    segments: Vec<CodeSegment>,
    clock_rate: Option<u32>,
    last_command: String,
    breakpoints: [u8; 64 * 1024],
    broken: bool,
    step: bool,
}

impl<'a> VirtualMachine<'a> {
    pub fn new<CR>(cpu: Cpu,
                   clock_rate: CR,
                   ttf_context: &'a Sdl2TtfContext,
                   mut renderer: &mut Renderer,
                   font_file: &'a str)
                   -> VirtualMachine<'a>
        where CR: Into<Option<u32>>
    {
        let mut console = Console::new(ttf_context, renderer, font_file);

        writeln!(console, "Welcome to hakka. Type 'help' for instructions").unwrap();
        writeln!(console, "").unwrap();

        VirtualMachine {
            cpu: cpu,
            console: console,
            segments: Vec::new(),
            clock_rate: clock_rate.into(),
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

    pub fn render(&mut self, mut renderer: &mut Renderer) {
        self.console.render(renderer);
    }

    pub fn load_code_segments(&mut self, segments: Vec<CodeSegment>) {
        if segments.is_empty() {
            return;
        }
        self.segments = segments;
        for segment in &self.segments {
            self.cpu.load(&segment.code, segment.address).unwrap();
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
                    self.console.println("");
                    self.console
                        .println(format!("BREAKPOINT hit at {:04X}", self.cpu.registers.PC));
                    // We are supposed to pass the current timestamp to prevent the keys which are
                    // used to toggle the console from inputing text into the console. As no key
                    // is pressed to open the console in this instance, passing the time is not
                    // strictly necesarry
                    self.console.toggle(0);
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
                self.console.println("");
                self.console.println(format!("BREAKPOINT hit at {:04X}", self.cpu.registers.PC));
                self.console.toggle(0);
            }
        }
    }

    pub fn execute_command<S>(&mut self, cmd: S)
        where S: Into<String>
    {
        let mut input: String = cmd.into().trim().into();

        if input == "r" || input == "repeat" {
            input = self.last_command.clone();
        }

        let parts = input.split(' ').collect::<Vec<_>>();

        if input.is_empty() {
            if self.monitor.enabled {
                self.monitor.enabled = false;
            }
        } else if input.ends_with("^C") {
            self.monitor.enabled = false;
        } else if parts[0] == "clear" || parts[0] == "cls" {
            self.console.clear();
        } else if parts[0] == "help" {
            self.console.print_lines(HELPTEXT);
        } else if parts[0] == "source" {
            self.dump_disassembly();
        } else if parts[0] == "list" {
            self.dump_local_disassembly();
        } else if parts[0] == "monitor" || parts[0] == "mon" {
            if self.monitor.enabled {
                self.monitor.enabled = false;
            } else {
                self.enable_memory_monitor(&input);
            }
        } else if parts[0] == "memset" || parts[0] == "set" {
            if parts.len() < 3 {
                self.console
                    .println("ERR: Requires 2 arguments. Example: memset 0x00 0x01 to store 0x01 \
                              in 0x00.");
            } else if parts.len() == 3 {
                if let Ok(dst) = u16::from_str_radix(&parts[1].replace("0x", "")[..], 16) {
                    if let Ok(src) = u8::from_str_radix(&parts[2].replace("0x", "")[..], 16) {
                        self.cpu.memory[dst as usize] = src;
                    } else {
                        self.console.println("ERR: Unable to parse source byte value");
                    }
                } else {
                    self.console.println("ERR: Unable to parse destination byte value");
                }
            } else if let Ok(mut dst) = usize::from_str_radix(&parts[1].replace("0x", "")[..], 16) {
                for p in &parts[2..] {
                    if let Ok(byte) = u8::from_str_radix(&p.replace("0x", "")[..], 16) {
                        self.cpu.memory[dst] = byte;
                        dst += 0x01;
                    } else {
                        self.console.println("ERR: Unable to parse source byte value");
                    }
                }
            } else {
                self.console.println("ERR: Unable to parse destination byte value");
            }
        } else if parts[0] == "memdmp" || parts[0] == "dmp" {
            // 1 argument assumes 1 memory "page"
            if parts.len() == 2 {
                if let Ok(page_number) = parts[1].parse() {

                    self.dump_memory_page(page_number);
                } else {
                    self.console.println("ERR: Unable to parse memory page");
                }
            } else if parts.len() == 3 {
                // A memory range instead
                if let Ok(start) = u16::from_str_radix(&parts[1].replace("0x", "")[..], 16) {
                    if let Ok(end) = u16::from_str_radix(&parts[2].replace("0x", "")[..], 16) {
                        self.dump_memory_range(start, end);
                    } else {
                        self.console.println("ERR: Unable to parse end address value");
                    }
                } else {
                    self.console.println("ERR: Unable to parse start address value");
                }
            }
        } else if parts[0] == "registers" || parts[0] == "reg" {
            self.dump_registers();
        } else if parts[0] == "flags" {
            self.dump_flags();
        } else if parts[0] == "break" || parts[0] == "b" {
            // 1 argument assumes 1 memory "page"
            if parts.len() == 2 {
                if let Ok(addr) = usize::from_str_radix(&parts[1][..], 16) {
                    if addr <= u16::max_value() as usize {
                        if self.breakpoints[addr] > 0 {
                            self.console.println(format!("Removed breakpoint at {:04X}", addr));
                            self.breakpoints[addr] = 0;
                        } else {
                            self.console.println(format!("Added breakpoint at {:04X}", addr));
                            self.breakpoints[addr] = 1;
                        }
                    } else {
                        self.console.println("ERR: Value outside addressable range.");
                    }
                } else {
                    self.console.println("ERR: Unable to parse breakpoint address");
                }
            } else {
                self.broken = true;
                self.console.println("Execution stopped");
            }
        } else if parts[0] == "continue" || parts[0] == "c" {
            self.broken = false;
            self.console.println("Execution resumed");
        } else if parts[0] == "step" || parts[0] == "s" {
            self.broken = true;
            self.step = true;
        } else {
            self.console.println("Unknown command");
        }

        // Don't assign a blank command as a last command
        if !input.is_empty() {
            self.last_command = input.clone();
        }
    }

    fn enable_memory_monitor(&mut self, input: &str) {
        let parts: Vec<&str> = input.split(' ').collect();
        if parts.len() < 3 {
            self.console.println("ERR: Requires 2 arguments. Example: monitor 0x00 0xFF");
        } else {
            let start = usize::from_str_radix(&parts[1].replace("0x", "")[..], 16).unwrap();
            let end = usize::from_str_radix(&parts[2].replace("0x", "")[..], 16).unwrap();

            self.monitor.start_addr = start;
            self.monitor.end_addr = end;
            self.monitor.enabled = true;
        }
    }

    pub fn is_debugging(&self) -> bool {
        self.broken
    }

    fn dump_disassembly(&mut self) {
        self.console.println("");

        for segment in &self.segments {
            self.console.println(format!(".ORG ${:04X}", segment.address));
            let disassembler = Disassembler::with_offset(segment.address);
            let pairs = disassembler.disassemble_with_addresses(&segment.code);
            let lines = self.highlight_lines(self.cpu.registers.PC as usize,
                                             pairs,
                                             segment.address,
                                             false);
            self.console.print_lines(lines.join(""));
        }
        self.console.println("");
    }

    fn dump_local_disassembly(&mut self) {
        self.console.println("");

        let result = {
            let pc = self.cpu.registers.PC as usize;
            let local_segment = self.get_local_segment(pc);
            let disassembler = Disassembler::with_offset(local_segment.address);
            let pairs = disassembler.disassemble_with_addresses(&local_segment.code);
            self.highlight_lines(pc, pairs, local_segment.address, true)
        };
        self.console.print_lines(result.join(""));
        self.console.println("");
    }

    pub fn dump_memory_page(&mut self, page: usize) {
        let mut addr = page * 0x100;
        for chunk in self.cpu.memory[page * 0x100..(page * 0x100) + 0x100].chunks(8) {
            self.console.print(format!("{:04X}: ", addr));
            for b in chunk {
                self.console.print(format!("{:02X} ", *b));
            }
            self.console.wrap_line();
            addr += 0x08;
        }
        self.console.println("");
    }

    pub fn dump_memory(&mut self) {
        for chunk in self.cpu.memory[self.monitor.start_addr..self.monitor.end_addr + 0x01]
            .chunks(8) {
            for b in chunk {
                self.console.print(format!("{:02X} ", *b));
            }
            self.console.wrap_line();
        }
    }

    fn dump_memory_range(&mut self, start: u16, end: u16) {
        let start = start as usize;
        let end = end as usize;
        for chunk in self.cpu.memory[start..end + 0x01].chunks(8) {
            for b in chunk {
                self.console.print(format!("{:02X} ", *b));
            }
            self.console.wrap_line();
        }
        self.console.println("");
    }

    fn dump_registers(&mut self) {
        self.console.println("");
        self.console.println(format!("A: {} ({:04X})", self.cpu.registers.A, self.cpu.registers.A));
        self.console.println(format!("X: {} ({:04X})", self.cpu.registers.X, self.cpu.registers.X));
        self.console.println(format!("Y: {} ({:04X})", self.cpu.registers.Y, self.cpu.registers.Y));
        self.console.println(format!("PC: {} ({:04X})",
                                     self.cpu.registers.PC,
                                     self.cpu.registers.PC));
        self.console.println(format!("S: {} ({:04X})",
                                     self.cpu.stack.pointer,
                                     self.cpu.stack.pointer));
    }

    fn dump_flags(&mut self) {
        self.console.println("");
        self.console.println(format!("Carry: {}", self.cpu.flags.carry));
        self.console.println(format!("Zero: {}", self.cpu.flags.zero));
        self.console.println(format!("Interrupts disabled: {}", self.cpu.flags.interrupt_disabled));
        self.console.println(format!("Decimal mode: {}", self.cpu.flags.decimal));
        self.console.println(format!("Break: {}", self.cpu.flags.breakpoint));
        self.console.println(format!("Overflow: {}", self.cpu.flags.overflow));
        self.console.println(format!("Sign: {}", self.cpu.flags.sign));
        self.console.println(format!("Unused: {}", self.cpu.flags.unused));
        self.console.println("");
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
