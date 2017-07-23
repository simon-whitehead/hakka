
extern crate byteorder;
extern crate find_folder;
extern crate rs6502;
extern crate sdl2;
extern crate vm;

mod button;
mod keypad;
mod lcd;

use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};

use find_folder::Search;

use sdl2::event::Event;
use sdl2::keyboard::*;
use sdl2::pixels::Color;
use sdl2::render::Renderer;

use rs6502::{Assembler, CodeSegment, Cpu};
use vm::{VirtualMachine, GameCore};

use keypad::Keypad;

const FPS_STEP: u32 = 1000 / 60;

fn main() {
    let window_width = 1280;
    let window_height = 720;

    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("hakka - 001 - The Door", window_width, window_height)
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let local = Search::Parents(3).for_folder("001-the-door").unwrap();
    let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

    let default_font = assets.join("FantasqueSansMono-Bold.ttf");
    let mut game_core = GameCore::new(1,
                                      &ttf_context,
                                      &mut renderer,
                                      default_font.to_str().unwrap());

    let segments = assemble(local.join("level.asm"));
    game_core.vm.load_code_segments(segments);
    game_core.vm.cpu.reset();
    let mask = ((((0x0A - 0b11) * (0b10 * 0x4)) - (0x31)) as u32) *
               (((((0x01 << 0x03) | 0b1) * 0x55555556 as u64) >> 0x20) as u16 *
                (((0x01 << 0b11) as u8).pow(0b10) as u16) as u16 - 1) as u32;
    game_core.vm.cpu.memory[0b1111000000000000] = mask.to_string().as_bytes()[0] - ('0' as u8);
    game_core.vm.cpu.memory[0b1111000000000001] = mask.to_string().as_bytes()[1] - ('0' as u8);
    game_core.vm.cpu.memory[0b1111000000000010] = mask.to_string().as_bytes()[2] - ('0' as u8);
    game_core.vm.cpu.memory[0b1111000000000011] = mask.to_string().as_bytes()[3] - ('0' as u8);
    game_core.vm.cpu.flags.interrupt_disabled = false;

    let mut keypad = Keypad::new(&ttf_context, &mut renderer);

    let mut events = sdl_context.event_pump().unwrap();

    let mut last_fps = 0;
    let mut monitor_last = 0;

    'running: loop {

        for event in events.poll_iter() {
            game_core.process_event(&event);
            keypad.process_event(&event, &ttf_context, &mut renderer, &mut game_core.vm.cpu);

            if !game_core.vm.console.visible {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyDown { keycode, .. } => {
                        match keycode {
                            Some(Keycode::Escape) => break 'running,
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }

        keypad.process(&ttf_context, &mut renderer, &mut game_core.vm.cpu);

        let now = sdl_context.timer().unwrap().ticks();
        let delta = now - last_fps;
        if delta < FPS_STEP {
            sdl_context.timer().unwrap().delay(FPS_STEP - delta);
        } else {
            game_core.update();

            renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
            renderer.clear();

            // Render game here
            keypad.render(&mut renderer);
            game_core.vm.render(&mut renderer);
            renderer.present();
            last_fps = now;
        }

        // Dump the CPU memory at 1 second intervals if the monitor is enabled
        let delta = now - monitor_last;
        if delta > 1000 && game_core.vm.monitor.enabled {
            game_core.vm.dump_memory();
            monitor_last = now;
        }
    }
}

fn assemble<P>(path: P) -> Vec<CodeSegment>
    where P: AsRef<Path>
{
    let mut assembler = Assembler::new();
    assembler.assemble_file(path, 0xC000).unwrap()
}