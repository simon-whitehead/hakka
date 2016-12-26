
extern crate byteorder;
extern crate find_folder;
extern crate app_dirs;
extern crate rustc_serialize;
extern crate rs6502;
extern crate sdl2;
extern crate vm;

mod ship;

use std::path::Path;

use byteorder::{ByteOrder, LittleEndian};

use find_folder::Search;

use app_dirs::{AppInfo, AppDataType};

use sdl2::event::Event;
use sdl2::keyboard::*;
use sdl2::image::LoadTexture;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, TextureQuery};

use rs6502::{Assembler, CodeSegment, Cpu};
use vm::{Console, Position, Text, VirtualMachine, Configuration};

const FPS_STEP: u32 = 1000 / 60;
const APP_INFO: AppInfo = AppInfo { name: "hakka", author: "simon-whitehead" };
const CONFIG_FILE: &'static str = "config.json";

fn main() {
    let config_root = app_dirs::app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
    let config_file = {
        let mut config_file = config_root.clone();
        config_file.push(CONFIG_FILE);
        config_file
    };

    if !config_file.exists() {
        let default_config = Configuration::default();
        default_config.store(&config_file).unwrap();
    }

    let config = Configuration::load(&config_file).unwrap();

    let window_width = 1280;
    let window_height = 720;

    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("hakka", window_width, window_height)
        .build()
        .unwrap();

    let (window_width, _) = window.size();

    let mut renderer = window.renderer()
        .accelerated()
        .build()
        .unwrap();

    let local = Search::Parents(3).for_folder("training-1").unwrap();
    let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

    let ship_texture = renderer.load_texture(&assets.join("ship.png")).unwrap();
    let ship_flame_texture = renderer.load_texture(&assets.join("ship-flame.png"))
        .unwrap();

    let font = assets.join("FantasqueSansMono-Bold.ttf");

    let finish_text = Text::new(&ttf_context,
                                &mut renderer,
                                "FINISH",
                                Position::HorizontalCenter((window_width / 2) as i32, 25),
                                56,
                                Color::RGBA(0, 0, 0, 255),
                                font.to_str().unwrap());

    let win_text = Text::new(&ttf_context,
                             &mut renderer,
                             "PASSED",
                             Position::HorizontalCenter((window_width / 2) as i32, 330),
                             64,
                             Color::RGBA(0, 0, 0, 255),
                             font.to_str().unwrap());

    let TextureQuery { width: ship_width, .. } = ship_texture.query();
    let cpu = init_cpu(&mut renderer, ship_width);
    let segments = assemble(local.join("level.asm"));
    let console = Console::new(&ttf_context, &mut renderer, font.to_str().unwrap(), &config);
    let mut vm = VirtualMachine::new(cpu, 150, console);
    vm.load_code_segments(segments);

    let mut events = sdl_context.event_pump().unwrap();

    let mut level_complete = false;
    let mut ship = ship::Ship::new(ship_texture,
                                   ship_flame_texture,
                                   Position::HorizontalCenter((window_width / 2) as i32, 500));
    let mut last_fps = 0;
    let mut monitor_last = 0;

    'running: loop {

        for event in events.poll_iter() {
            if vm.console.visible {
                vm.console.process(&event);
            } else {
                match event {
                    Event::Quit { .. } => break 'running,
                    Event::KeyUp { keycode, .. } => {
                        match keycode {
                            Some(Keycode::Up) |
                            Some(Keycode::Down) => {
                                vm.cpu.memory[0x04] = 0;
                            }
                            _ => (),
                        }
                    }
                    Event::KeyDown { keycode, scancode, timestamp, keymod, .. } => {
                        if !keymod.intersects(LALTMOD | LCTRLMOD | LSHIFTMOD | RALTMOD | RCTRLMOD |
                                        RSHIFTMOD) {
                            if scancode.is_some() && scancode.unwrap() == config.get_scancode() {
                                vm.console.toggle(timestamp);
                            }
                        }

                        match keycode {
                            Some(Keycode::Escape) => break 'running,

                            // Movement
                            Some(Keycode::Up) => {
                                vm.cpu.memory[0x04] = 38;
                            }
                            Some(Keycode::Down) => {
                                vm.cpu.memory[0x04] = 40;
                            }
                            _ => (),
                        }
                    }
                    _ => (),
                }
            }
        }

        if !level_complete {
            if let Some(cmd) = vm.console.try_process_command() {
                vm.execute_command(cmd);
            }
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

            // Rendering only the background when interrupts are disabled results in a horrible
            // flickering; therefore only render when we're either in single stepping mode or
            // interrupts are enabled
            if vm.is_debugging() || !vm.cpu.flags.interrupt_disabled {
                renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
                renderer.clear();

                // Render complete game screen only if interrupts are enabled
                if !vm.cpu.flags.interrupt_disabled {
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

fn init_cpu(renderer: &mut Renderer, ship_width: u32) -> Cpu {
    let mut cpu = Cpu::new();
    cpu.flags.interrupt_disabled = false;

    LittleEndian::write_u16(&mut cpu.memory[0..],
                            renderer.window().unwrap().size().0 as u16 / 2 -
                            (ship_width as u16 / 2));
    cpu.memory[0x02] = 0xFF;
    cpu.memory[0x03] = 0x01;
    cpu.memory[0x05] = 0x05;
    cpu.memory[0x06] = 0x00;

    cpu
}

fn draw_text_background(renderer: &mut Renderer, color: Color, y: i32) {
    let width = renderer.window().unwrap().size().0;
    renderer.set_draw_color(color);
    renderer.fill_rect(Rect::new(0, y, width, 120)).unwrap();
}

fn draw_finish_background(renderer: &mut Renderer) {
    draw_text_background(renderer, Color::RGB(0, 144, 192), 0);
}

fn draw_passed_background(renderer: &mut Renderer) {
    draw_text_background(renderer, Color::RGB(0, 255, 0), 300);
}
