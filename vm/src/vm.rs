
use rs6502::{CodeSegment, Cpu, Disassembler};
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;
use console::Console;
use std::io::Write;
use std::ops::Range;

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
            // This helps check for breakpoints on the entrypoint of an interrupt
            // request handler
            if self.cpu.flags.interrupt_disabled && !self.broken {
                self.check_breakpoint();
            }
            while (n < clock_rate && !self.broken) || self.step {
                n += self.cpu.step().expect("SEGFAULT") as u32;
                self.check_breakpoint();
                // If we stepped, dump the local disassembly
                if self.step {
                    self.dump_local_disassembly();
                }
                self.step = false;
            }
        } else {
            // This helps check for breakpoints on the entrypoint of an interrupt
            // request handler
            if self.cpu.flags.interrupt_disabled && !self.broken {
                self.check_breakpoint();
            }
            self.cpu.step().expect("SEGFAULT");
            if self.step {
                self.dump_local_disassembly();
            }
            self.step = false;
            self.check_breakpoint();
        }
    }

    fn check_breakpoint(&mut self) {
        if self.breakpoints[self.cpu.registers.PC as usize] > 0 {
            self.broken = true;
            writeln!(self.console, "").unwrap();
            writeln!(self.console,
                     "BREAKPOINT hit at {:04x}",
                     self.cpu.registers.PC)
                .unwrap();
            self.dump_local_disassembly();
            // We are supposed to pass the current timestamp to prevent the keys which are
            // used to toggle the console from inputing text into the console. As no key
            // is pressed to open the console in this instance, passing the time is not
            // strictly necesarry
            self.console.toggle(0);
        }
    }

    pub fn enable_memory_monitor(&mut self, range: Range<usize>) {
        self.monitor.start_addr = range.start;
        self.monitor.end_addr = range.end;
        self.monitor.enabled = true;
    }
    pub fn is_memory_monitor_enabled(&self) -> bool {
        self.monitor.enabled
    }
    pub fn disable_memory_monitor(&mut self) {
        self.monitor.enabled = false;
    }

    pub fn is_debugging(&self) -> bool {
        self.broken
    }

    pub fn break_execution(&mut self) {
        self.broken = true;
    }
    pub fn continue_execution(&mut self) {
        self.broken = false;
    }
    pub fn step_execution(&mut self) {
        self.broken = true;
        self.step = true;
    }
    pub fn toggle_breakpoint(&mut self, address: usize) -> bool {
        if self.breakpoints[address] > 0 {
            self.breakpoints[address] = 0;
            return false;
        } else {
            self.breakpoints[address] = 1;
            return true;
        }
    }

    pub fn dump_disassembly(&mut self) {
        writeln!(self.console, " ").unwrap();

        for segment in &self.segments {
            writeln!(self.console, ".ORG ${:04X}", segment.address).unwrap();
            let disassembler = Disassembler::with_offset(segment.address);
            let pairs = disassembler.disassemble_with_addresses(&segment.code);
            let lines = self.highlight_lines(self.cpu.registers.PC as usize,
                                             pairs,
                                             segment.address,
                                             false);
            for line in lines {
                write!(self.console, "{}", line).unwrap();
            }
        }

        writeln!(self.console, " ").unwrap();
    }

    pub fn dump_local_disassembly(&mut self) {
        writeln!(self.console, " ").unwrap();

        let result = {
            let pc = self.cpu.registers.PC as usize;
            let local_segment = self.get_local_segment(pc);
            let disassembler = Disassembler::with_offset(local_segment.address);
            let pairs = disassembler.disassemble_with_addresses(&local_segment.code);
            self.highlight_lines(pc, pairs, local_segment.address, true)
        };
        for line in result {
            write!(self.console, "{}", line).unwrap();
        }
        writeln!(self.console, "").unwrap();
    }

    pub fn dump_memory_page(&mut self, page: usize) {
        let mut addr = page * 0x100;
        for chunk in self.cpu.memory[page * 0x100..(page * 0x100) + 0x100].chunks(8) {
            write!(self.console, "{:04X}: ", addr).unwrap();
            for b in chunk {
                write!(self.console, "{:02X} ", *b).unwrap();
            }
            writeln!(self.console, "").unwrap();
            addr += 0x08;
        }
        writeln!(self.console, "").unwrap();
    }

    pub fn dump_memory(&mut self) {
        for chunk in self.cpu.memory[self.monitor.start_addr..self.monitor.end_addr + 0x01]
            .chunks(8) {
            for b in chunk {
                write!(self.console, "{:02X} ", *b).unwrap();
            }
            writeln!(self.console, "").unwrap();
        }
    }

    pub fn dump_memory_range(&mut self, start: usize, end: usize) {
        for chunk in self.cpu.memory[start..end + 0x01].chunks(8) {
            for b in chunk {
                write!(self.console, "{:02X} ", *b).unwrap();
            }
            writeln!(self.console, "").unwrap();
        }
        writeln!(self.console, "").unwrap();
    }

    pub fn dump_registers(&mut self) {
        writeln!(self.console, " ").unwrap();
        writeln!(self.console,
                 "A: {} ({:04X})",
                 self.cpu.registers.A,
                 self.cpu.registers.A)
            .unwrap();
        writeln!(self.console,
                 "X: {} ({:04X})",
                 self.cpu.registers.X,
                 self.cpu.registers.X)
            .unwrap();
        writeln!(self.console,
                 "Y: {} ({:04X})",
                 self.cpu.registers.Y,
                 self.cpu.registers.Y)
            .unwrap();
        writeln!(self.console,
                 "PC: {} ({:04X})",
                 self.cpu.registers.PC,
                 self.cpu.registers.PC)
            .unwrap();
        writeln!(self.console,
                 "S: {} ({:04X})",
                 self.cpu.stack.pointer,
                 self.cpu.stack.pointer)
            .unwrap();
        writeln!(self.console, " ").unwrap();
    }

    pub fn dump_flags(&mut self) {
        writeln!(self.console, " ").unwrap();
        writeln!(self.console, "Carry: {}", self.cpu.flags.carry).unwrap();
        writeln!(self.console, "Zero: {}", self.cpu.flags.zero).unwrap();
        writeln!(self.console,
                 "Interrupts disabled: {}",
                 self.cpu.flags.interrupt_disabled)
            .unwrap();
        writeln!(self.console, "Decimal mode: {}", self.cpu.flags.decimal).unwrap();
        writeln!(self.console, "Break: {}", self.cpu.flags.breakpoint).unwrap();
        writeln!(self.console, "Overflow: {}", self.cpu.flags.overflow).unwrap();
        writeln!(self.console, "Sign: {}", self.cpu.flags.sign).unwrap();
        writeln!(self.console, "Unused: {}", self.cpu.flags.unused).unwrap();
        writeln!(self.console, " ").unwrap();
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
