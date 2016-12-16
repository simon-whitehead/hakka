extern crate rs6502;
extern crate sdl2;

mod input;
mod ship;

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use std::time::Duration;

use rs6502::{Assembler, Cpu, Disassembler};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::image::LoadTexture;
use sdl2::Sdl;

const FPS_STEP: u32 = 1000 / 60;

fn main() {
    let (tx, rx) = channel();

    input::handle(tx);

    let bytecode = assemble("level.asm");
    let mut cpu = init_cpu(&bytecode);

    let sdl_context = sdl2::init().unwrap();
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

    let mut events = sdl_context.event_pump().unwrap();

    let mut ship = ship::Ship::new(ship_texture);

    let mut last_fps = 0;
    let mut monitor_enabled = false;
    let mut monitor_last = 0;

    'running: loop {
        cpu.step();

        if let Ok(input) = rx.try_recv() {
            let input = input.trim();
            if input == "exit" {
                break 'running;
            }

            if input == "list" {
                std::io::stdout().write(b"\n");
                std::io::stdout().write(b"-- Disassembly --\n");

                let mut disassembler = Disassembler::with_offset(0xC000);
                let asm = disassembler.disassemble(&bytecode[..]);

                std::io::stdout().write(asm.as_bytes());
                std::io::stdout().write(b"\nHAKKA> ");
                std::io::stdout().flush();
            }

            if input == "monitor" {
                monitor_enabled = !monitor_enabled;
            }

            if input.as_bytes().starts_with(b"memset") {
                let parts: Vec<&str> = input.split(" ").collect();
                if parts.len() < 3 {
                    println!("ERR: Requires 2 arguments. Example: memset 0x00 0x01 to store 0x01 \
                              in 0x00.");
                } else {
                    let dst = u16::from_str_radix(&parts[1].replace("0x", "")[..], 16).unwrap();
                    let src = u8::from_str_radix(&parts[2].replace("0x", "")[..], 16).unwrap();

                    cpu.memory[dst as usize] = src;
                }
            }
        }

        ship.process(&cpu.memory[..]);

        for event in events.poll_iter() {
            match event {
                Event::Quit { .. } => break 'running,
                Event::KeyDown { keycode: Option::Some(Keycode::Right), .. } => {
                    cpu.memory[0x04] = 39
                }
                Event::KeyDown { keycode: Option::Some(Keycode::Left), .. } => {
                    cpu.memory[0x04] = 37
                }
                Event::KeyUp { keycode: Option::Some(Keycode::Right), .. } => cpu.memory[0x04] = 0,
                Event::KeyUp { keycode: Option::Some(Keycode::Left), .. } => cpu.memory[0x04] = 0,
                Event::KeyDown { keycode: Option::Some(Keycode::Escape), .. } => break 'running,
                _ => (),
            }
        }

        let now = sdl_context.timer().unwrap().ticks();
        let delta = now - last_fps;
        if delta < FPS_STEP {
            sdl_context.timer().unwrap().delay(FPS_STEP - delta);
        } else {
            if !cpu.flags.interrupt_disabled {
                // Render stuff here
                renderer.clear();
                ship.render(&mut renderer);
                renderer.present();
                last_fps = now;
            }
        }

        let delta = now - monitor_last;
        if delta > 1000 && monitor_enabled {
            for b in &cpu.memory[0x00..0xA] {
                print!("{:02X} ", *b);
            }
            println!("");
            monitor_last = now;
        }

        // thread::sleep(Duration::from_millis(10));
    }
}

fn assemble<P>(path: P) -> Vec<u8>
    where P: AsRef<Path>
{
    let mut assembler = Assembler::new();
    assembler.assemble_file("level.asm").unwrap()
}

fn init_cpu(bytecode: &[u8]) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.load(&bytecode[..], None);
    cpu.flags.interrupt_disabled = false;

    cpu.memory[0x02] = 0x80;
    cpu.memory[0x03] = 0x00;
    cpu.memory[0x05] = 0x19;
    cpu.memory[0x06] = 0x00;

    cpu
}