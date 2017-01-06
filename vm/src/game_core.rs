
use std::io::Write;

use vm::VirtualMachine;
use console::Console;
use command::{CommandSystem, Command};

use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::event::Event;

use rs6502::Cpu;

pub struct GameCore<'a> {
    pub vm: VirtualMachine<'a>,
    pub command_system: CommandSystem,
}

impl<'a> GameCore<'a> {
    pub fn new(ttf_context: &'a Sdl2TtfContext,
               mut renderer: &mut Renderer,
               font_file: &'a str)
               -> GameCore<'a>
   {
        let mut cpu = Cpu::new();
        let mut vm = VirtualMachine::new(cpu, 150, &ttf_context, &mut renderer, font_file);

        GameCore {
            vm: vm,
            command_system: CommandSystem::new(),
        }
    }

    pub fn process_event(&mut self, event: &Event) {
        self.vm.console.process(&event);
    }

    pub fn update(&mut self) {
        if let Some(cmd) = self.vm.console.get_next_command() {
            let success = self.command_system.execute(cmd, &mut self.vm);
            if !success {
                writeln!(self.vm.console, "Command not recognized, type 'help' for a list of commands").unwrap();
            }
        }
        self.vm.cycle();
    }
}
