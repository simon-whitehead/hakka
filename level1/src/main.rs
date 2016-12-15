extern crate rs6502;
extern crate sdl2;

mod ship;
mod timer;

use std::io::{self, BufRead, Write};
use std::path::Path;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

use rs6502::{Assembler, Cpu, Disassembler};

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::image::LoadTexture;
use sdl2::Sdl;

use ::timer::FrameTimer;

fn main() {
    let (tx, rx) = channel();

    thread::spawn(move || {
        loop {
            std::io::stdout().write(b"HAKKA> ");
            std::io::stdout().flush();

            let mut line = String::new();
            let stdin = io::stdin();
            stdin.lock().read_line(&mut line).expect("Could not read line");
            tx.send(line).unwrap();
        }
    });

    let mut cpu = Cpu::new();
    let mut assembler = Assembler::new();
    let bytecode = assembler.assemble_file("level.asm").unwrap();
    cpu.load(&bytecode[..], None);
    cpu.flags.interrupt_disabled = false;

    cpu.memory[0x02] = 0x80;
    cpu.memory[0x03] = 0x00;
    cpu.memory[0x05] = 0x19;
    cpu.memory[0x06] = 0x00;

    let mut timer = FrameTimer::new(1000 / 120, 0, 0, 0);

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

    'running: loop {

        cpu.step_n(4);
        if let Ok(input) = rx.try_recv() {
            let input = input.trim();
            if input == "exit" {
                break 'running;
            }

            if input == "list" {
                std::io::stdout().write(b"\n");
                std::io::stdout().write(b"-- Disassembly --\n");

                let mut disassembler = Disassembler::new();
                let asm = disassembler.disassemble(&bytecode[..]);

                std::io::stdout().write(asm.as_bytes());
                std::io::stdout().write(b"\nHAKKA> ");
                std::io::stdout().flush();
            }
        }

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

        if frame_cap(&sdl_context, &mut timer) {
            ship.process(&cpu.memory[..]);

            if !cpu.flags.interrupt_disabled {
                // Render stuff here
                renderer.clear();
                ship.render(&mut renderer);
                renderer.present();
            }

        }

        if cpu.finished() {
            cpu.reset();
        }

        // thread::sleep(Duration::from_millis(10));
    }
}

fn frame_cap(sdl_context: &Sdl, timer: &mut FrameTimer) -> bool {
    let now = sdl_context.timer().unwrap().ticks();
    let delta = now - timer.prev;
    let elapsed = delta as f64 / 1000.0;

    timer.ticks = now;

    // Wait until 1/60th of a second has passed since we last called this
    if delta < timer.interval {
        sdl_context.timer().unwrap().delay(timer.interval - delta);
        return false;
    }

    timer.prev = now;
    timer.fps += 1;

    timer.elapsed = elapsed;

    if now - timer.last_second > 1000 {
        // Store our current FPS
        timer.last_fps = timer.fps;
        timer.last_second = now;
        timer.fps = 0;
    }

    true
}