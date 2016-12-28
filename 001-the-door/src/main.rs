
extern crate byteorder;
extern crate find_folder;
extern crate rs6502;
extern crate sdl2;
extern crate vm;

use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};

use find_folder::Search;

use sdl2::event::Event;
use sdl2::keyboard::*;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, TextureQuery};

use rs6502::{Assembler, CodeSegment, Cpu};
use vm::{Position, Text, VirtualMachine};

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

    let (window_width, _) = window.size();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let local = Search::Parents(3).for_folder("001-the-door").unwrap();
    let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

    let font = assets.join("FantasqueSansMono-Bold.ttf");

    let cpu = init_cpu(&mut renderer);
    let segments = assemble(local.join("level.asm"));
    let mut vm = VirtualMachine::new(cpu,
                                     150,
                                     &ttf_context,
                                     &mut renderer,
                                     font.to_str().unwrap());
    vm.load_code_segments(segments);

    let mut events = sdl_context.event_pump().unwrap();

    let mut last_fps = 0;
    let mut monitor_last = 0;

    'running: loop {

        for event in events.poll_iter() {
            vm.console.process(&event);

            if !vm.console.visible {
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

        if let Some(cmd) = vm.console.try_process_command() {
            vm.execute_command(cmd);
        }

        let now = sdl_context.timer().unwrap().ticks();
        let delta = now - last_fps;
        if delta < FPS_STEP {
            sdl_context.timer().unwrap().delay(FPS_STEP - delta);
        } else {
            vm.cycle();

            // Rendering only the background when interrupts are disabled results in a horrible
            // flickering; therefore only render when we're either in single stepping mode or
            // interrupts are enabled
            if vm.is_debugging() || !vm.cpu.flags.interrupt_disabled {
                renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
                renderer.clear();

                // Render complete game screen only if interrupts are enabled
                if !vm.cpu.flags.interrupt_disabled {
                    // Render game here
                }
                vm.render(&mut renderer);
                renderer.present();
                last_fps = now;
            }
        }

        // Dump the CPU memory at 1 second intervals if the monitor is enabled
        let delta = now - monitor_last;
        if delta > 1000 && vm.monitor.enabled {
            vm.dump_memory();
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

fn init_cpu(renderer: &mut Renderer) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.flags.interrupt_disabled = false;

    cpu
}