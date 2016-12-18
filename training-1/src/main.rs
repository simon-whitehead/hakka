extern crate rs6502;
extern crate sdl2;
extern crate vm;

mod ship;

use std::path::Path;

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

    let window = video_subsystem.window("Hakka", 400, 720)
        .position_centered()
        .build()
        .unwrap();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let ship_texture = renderer.load_texture(Path::new("ship.png")).unwrap();
    let ship_flame_texture = renderer.load_texture(Path::new("ship-flame.png")).unwrap();

    let mut win_font = ttf_context.load_font(Path::new("FantasqueSansMono-Bold.ttf"), 64).unwrap();
    win_font.set_style(sdl2::ttf::STYLE_BOLD);

    let mut finish_font = ttf_context.load_font(Path::new("FantasqueSansMono-Bold.ttf"), 64)
        .unwrap();
    finish_font.set_style(sdl2::ttf::STYLE_BOLD);

    let win_message_surface = win_font.render("You win!")
        .blended(Color::RGBA(0, 153, 192, 255))
        .unwrap();
    let finish_banner_surface = finish_font.render("FINISH")
        .blended_wrapped(Color::RGBA(0, 0, 0, 255), 56)
        .unwrap();
    let win_message_texture = renderer.create_texture_from_surface(&win_message_surface)
        .unwrap();
    let finish_banner_texture = renderer.create_texture_from_surface(&finish_banner_surface)
        .unwrap();
    let mut events = sdl_context.event_pump().unwrap();

    let mut ship = ship::Ship::new(ship_texture, ship_flame_texture);
    let mut last_fps = 0;
    let mut monitor_last = 0;
    let TextureQuery { width: win_width, height: win_height, .. } = win_message_texture.query();
    let TextureQuery { width: finish_width, height: finish_height, .. } =
        finish_banner_texture.query();

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

        if ship.y < 0xA0 {
            renderer.clear();
            renderer.copy(&win_message_texture,
                      None,
                      Some(Rect::new(75, 200, win_width, win_height)))
                .unwrap();
            renderer.present();
        } else {
            vm.try_execute_command();
            ship.process(&vm.cpu.memory[..]);

            // Pull the ship back so it can't go past a certain spot
            if ship.y <= 0x190 && vm.cpu.memory[0x04] != 0 {
                vm.cpu.memory[0x02] = 0x90;
                vm.cpu.memory[0x03] = 0x01;
            }

            let now = sdl_context.timer().unwrap().ticks();
            let delta = now - last_fps;
            if delta < FPS_STEP {
                sdl_context.timer().unwrap().delay(FPS_STEP - delta);
            } else {
                vm.cycle();
                if !vm.cpu.flags.interrupt_disabled {
                    renderer.clear();
                    renderer.set_draw_color(Color::RGB(0, 144, 192));
                    renderer.fill_rect(Rect::new(0, 0, 400, 120)).unwrap();
                    renderer.set_draw_color(Color::RGB(0, 0, 0));
                    renderer.copy(&finish_banner_texture,
                              None,
                              Some(Rect::new(100, 25, finish_width, finish_height)))
                        .unwrap();
                    if vm.cpu.memory[0x07] > 0 {
                        ship.render_flame(&mut renderer);
                    }
                    ship.render(&mut renderer);
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

    cpu.memory[0x00] = 0x90;
    cpu.memory[0x02] = 0xFF;
    cpu.memory[0x03] = 0x01;
    cpu.memory[0x05] = 0x05;
    cpu.memory[0x06] = 0x00;

    cpu
}