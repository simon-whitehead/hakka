extern crate rs6502;
extern crate sdl2;
extern crate vm;

mod ship;
mod text;

use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use rs6502::{Assembler, Cpu};
use vm::VirtualMachine;

const FPS_STEP: u32 = 1000 / 60;
const WINDOW_WIDTH: u32 = 400;
const WINDOW_HEIGHT: u32 = 720;

fn main() {

    let bytecode = assemble("level.asm");
    let cpu = init_cpu(&bytecode);
    let mut vm = VirtualMachine::new(cpu, 0xC000, 150);

    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("hakka", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let ship_texture = renderer.load_texture(Path::new("ship.png")).unwrap();
    let ship_flame_texture = renderer.load_texture(Path::new("ship-flame.png")).unwrap();
    let finish_text = text::Text::new(&ttf_context,
                                      &mut renderer,
                                      "FINISH",
                                      100,
                                      25,
                                      56,
                                      Color::RGBA(0, 0, 0, 255),
                                      "FantasqueSansMono-Bold.ttf");

    let win_text = text::Text::new(&ttf_context,
                                   &mut renderer,
                                   "PASSED",
                                   100,
                                   330,
                                   64,
                                   Color::RGBA(0, 0, 0, 255),
                                   "FantasqueSansMono-Bold.ttf");

    let mut events = sdl_context.event_pump().unwrap();

    let mut level_complete = false;
    let mut ship = ship::Ship::new(ship_texture, ship_flame_texture);
    let mut last_fps = 0;
    let mut monitor_last = 0;

    'running: loop {

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Option::Some(Keycode::Up), .. } => {
                    vm.cpu.memory[0x04] = 38;
                }
                Event::KeyDown { keycode: Option::Some(Keycode::Down), .. } => {
                    vm.cpu.memory[0x04] = 40;
                }
                Event::KeyUp { keycode: Option::Some(Keycode::Up), .. } => {
                    vm.cpu.memory[0x04] = 0;
                }
                Event::KeyUp { keycode: Option::Some(Keycode::Down), .. } => {
                    vm.cpu.memory[0x04] = 0;
                }
                Event::KeyDown { keycode: Option::Some(Keycode::Escape), .. } => break 'running,
                _ => (),
            }
        }

        if !level_complete {
            vm.try_execute_command();
            ship.process(&vm.cpu.memory[..]);

            // Pull the ship back so it can't go past a certain spot
            if ship.y <= 0x190 && ship.y >= 0x100 && vm.cpu.memory[0x04] != 0 {
                vm.cpu.memory[0x02] = 0x90;
                vm.cpu.memory[0x03] = 0x01;
            }
        }

        let now = sdl_context.timer().unwrap().ticks();
        let delta = now - last_fps;
        if delta < FPS_STEP {
            sdl_context.timer().unwrap().delay(FPS_STEP - delta);
        } else {
            vm.cycle();
            if !vm.cpu.flags.interrupt_disabled {
                renderer.clear();
                if level_complete {
                    draw_passed_background(&mut renderer);
                    win_text.render(&mut renderer);
                }
                draw_finish_background(&mut renderer);
                finish_text.render(&mut renderer);
                if vm.cpu.memory[0x07] > 0 {
                    ship.render_flame(&mut renderer);
                }
                ship.render(&mut renderer);
                if ship.y <= 0x8C {
                    level_complete = true;
                }
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

fn assemble<P>(path: P) -> Vec<u8>
    where P: AsRef<Path>
{
    let mut assembler = Assembler::new();
    assembler.assemble_file(path, 0xC000).unwrap()
}

fn init_cpu(bytecode: &[u8]) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.load(&bytecode[..], None).unwrap();
    cpu.flags.interrupt_disabled = false;

    cpu.memory[0x00] = 0x90;
    cpu.memory[0x02] = 0xFF;
    cpu.memory[0x03] = 0x01;
    cpu.memory[0x05] = 0x05;
    cpu.memory[0x06] = 0x00;

    cpu
}

fn draw_text_background(renderer: &mut Renderer, color: Color, y: i32) {
    renderer.set_draw_color(color);
    renderer.fill_rect(Rect::new(0, y, WINDOW_WIDTH, 120)).unwrap();
}

fn draw_finish_background(renderer: &mut Renderer) {
    draw_text_background(renderer, Color::RGB(0, 144, 192), 0);
}

fn draw_passed_background(renderer: &mut Renderer) {
    draw_text_background(renderer, Color::RGB(0, 255, 0), 300);
}