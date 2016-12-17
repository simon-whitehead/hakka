extern crate rs6502;
extern crate sdl2;
extern crate vm;

mod ship;

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::mpsc::channel;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::TextureQuery;

use rs6502::{Assembler, Cpu};
use vm::VirtualMachine;

const FPS_STEP: u32 = 1000 / 60;

fn main() {

    let bytecode = assemble("level.asm");
    let cpu = init_cpu(&bytecode);
    let mut vm = VirtualMachine::new(cpu, 0xC000, 150);

    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Hakka", 1280, 400)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let ship_texture = renderer.load_texture(Path::new("ship.png")).unwrap();

    let mut win_font = ttf_context.load_font(Path::new("FantasqueSansMono-Bold.ttf"), 128).unwrap();
    win_font.set_style(sdl2::ttf::STYLE_BOLD);

    let mut finish_font = ttf_context.load_font(Path::new("FantasqueSansMono-Bold.ttf"), 64)
        .unwrap();
    finish_font.set_style(sdl2::ttf::STYLE_BOLD);

    let win_message_surface = win_font.render("You win!")
        .blended(Color::RGBA(0, 153, 192, 255))
        .unwrap();
    let finish_banner_surface = finish_font.render("F I N I S H")
        .blended_wrapped(Color::RGBA(0, 0, 0, 255), 56)
        .unwrap();
    let mut win_message_texture = renderer.create_texture_from_surface(&win_message_surface)
        .unwrap();
    let mut finish_banner_texture = renderer.create_texture_from_surface(&finish_banner_surface)
        .unwrap();
    let mut events = sdl_context.event_pump().unwrap();

    let mut ship = ship::Ship::new(ship_texture);
    let mut last_fps = 0;
    let mut monitor_last = 0;

    'running: loop {

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Option::Some(Keycode::Right), .. } => {
                    vm.cpu.memory[0x04] = 39
                }
                Event::KeyDown { keycode: Option::Some(Keycode::Left), .. } => {
                    vm.cpu.memory[0x04] = 37
                }
                Event::KeyUp { keycode: Option::Some(Keycode::Right), .. } => {
                    vm.cpu.memory[0x04] = 0
                }
                Event::KeyUp { keycode: Option::Some(Keycode::Left), .. } => {
                    vm.cpu.memory[0x04] = 0
                }
                Event::KeyDown { keycode: Option::Some(Keycode::Escape), .. } => break 'running,
                _ => (),
            }
        }

        if ship.x > 1280 - 0xFF {
            let TextureQuery { width, height, .. } = win_message_texture.query();
            renderer.clear();
            renderer.copy(&win_message_texture,
                          None,
                          Some(Rect::new(350, 128, width, height)));
            renderer.present();
        } else {
            vm.try_execute_command();
            ship.process(&vm.cpu.memory[..]);

            if ship.x >= 0x190 && ship.x <= 0x1AF && vm.cpu.memory[0x04] != 0 {
                vm.cpu.memory[0x00] = 0x90;
                vm.cpu.memory[0x01] = 0x01;
            }

            let now = sdl_context.timer().unwrap().ticks();
            let delta = now - last_fps;
            if delta < FPS_STEP {
                sdl_context.timer().unwrap().delay(FPS_STEP - delta);
            } else {
                vm.cycle();
                if !vm.cpu.flags.interrupt_disabled {
                    let TextureQuery { width, height, .. } = finish_banner_texture.query();
                    renderer.clear();
                    renderer.set_draw_color(Color::RGB(0, 144, 192));
                    renderer.fill_rect(Rect::new(1160, 0, 120, 400)).unwrap();
                    renderer.set_draw_color(Color::RGB(0, 0, 0));
                    renderer.copy(&finish_banner_texture,
                                  None,
                                  Some(Rect::new(1200, 0, width, height)));
                    ship.render(&mut renderer);
                    renderer.present();
                    last_fps = now;
                }
            }

            let delta = now - monitor_last;
            if delta > 1000 && vm.monitor.enabled {
                vm.dump_memory();
                monitor_last = now;
            }
        }
    }
}

fn assemble<P>(path: P) -> Vec<u8>
    where P: AsRef<Path>
{
    let mut assembler = Assembler::new();
    assembler.assemble_file(path).unwrap()
}

fn init_cpu(bytecode: &[u8]) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.load(&bytecode[..], None).unwrap();
    cpu.flags.interrupt_disabled = false;

    cpu.memory[0x02] = 0x80;
    cpu.memory[0x03] = 0x00;
    cpu.memory[0x05] = 0x05;
    cpu.memory[0x06] = 0x00;

    cpu
}