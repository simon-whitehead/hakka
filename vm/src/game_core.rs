
use std::io::Write;

use vm::VirtualMachine;
use command::{CommandSystem, UnblockEvent, CommandResult};

use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;
use sdl2::event::Event;
use sdl2::keyboard::{Keycode, LCTRLMOD, RCTRLMOD};

use rs6502::Cpu;

pub struct GameCore<'a> {
    pub vm: VirtualMachine<'a>,
    pub command_system: CommandSystem,
    unblock_event: Option<UnblockEvent>,
}

impl<'a> GameCore<'a> {
    pub fn new(ttf_context: &'a Sdl2TtfContext,
               mut renderer: &mut Renderer,
               font_file: &'a str)
               -> GameCore<'a>
   {
        let cpu = Cpu::new();
        let vm = VirtualMachine::new(cpu, 150, &ttf_context, &mut renderer, font_file);

        GameCore {
            vm: vm,
            command_system: CommandSystem::new(),
            unblock_event: None,
        }
    }

    pub fn process_event(&mut self, event: &Event) {
        if self.unblock_event.is_some() {
        }
        match *event {
            // Stop a blocking event
            Event::KeyDown { keycode, keymod, .. }
            if keycode == Some(Keycode::C) &&
               keymod.intersects(LCTRLMOD | RCTRLMOD) => {
                if let Some(ref unblock_event) = self.unblock_event {
                    unblock_event(&mut self.vm);
                }
                self.unblock_event = None;
            },
            // Let the console handle the event
            _ => self.vm.console.process(event)
        }
    }

    pub fn update(&mut self) {
        if let Some(cmd) = self.vm.console.get_next_command() {
            let (result, unblock_event) = self.command_system.execute(cmd, &mut self.vm);

            if let CommandResult::NotFound = result {
                writeln!(self.vm.console, "Command not recognized, type 'help' for a list of commands").unwrap();
            }

            if unblock_event.is_some() {
                self.unblock_event = unblock_event;
            } else {
                self.unblock_event = None;
            }
        }

        self.vm.cycle();
    }
}

