
use std;
use std::path::Path;
use std::io::Write;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::*;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Renderer, Texture, TextureQuery};
use sdl2::surface::Surface;
use sdl2::ttf::{Font, Sdl2TtfContext, STYLE_BOLD};

use app_dirs::*;

use position::Position;
use text::Text;
use config::{Configuration, ConfigError};

const APP_INFO: AppInfo = AppInfo { name: "hakka", author: "simon-whitehead" };
const CONFIG_FILE: &'static str = "config.json";

const BORDER_COLOR: Color = Color::RGBA(255, 255, 255, 64);

const PADDING: i32 = 10;

const FONT_COLOR: Color = Color::RGBA(45, 200, 45, 255);
const FONT_SIZE: u16 = 18;

pub struct Console<'a> {
    pub visible: bool,
    pub input_blocked: bool,
    visible_start_time: u32, /* Used to ensure that the KeyDown event that opens the console does not trigger text input */

    config: Configuration,

    font_file: &'a str,
    leader: Text,
    input_buffer: String,
    last_command: String,
    command_history: Vec<String>,
    history_position: usize,
    cursor_position: usize,
    buffer: Vec<String>,
    backbuffer_y: i32,
    texture: Texture,
    ttf_context: &'a Sdl2TtfContext,
    size: (u32, u32),
    font: Font<'a>,
}

impl<'a> Console<'a> {
    /// Creates a new empty Console
    pub fn new(ttf_context: &'a Sdl2TtfContext,
               mut renderer: &mut Renderer,
               font_file: &'a str)
               -> Console<'a> {

        let (width, height) = renderer.window().unwrap().size();
        let mut texture =
            renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, width / 2, height)
                .unwrap();
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..height {
                    for x in 0..width / 2 {
                        let x = x as usize;
                        let y = y as usize;
                        let offset = y * pitch + x * 4;
                        buffer[offset] = 182;
                        buffer[offset + 1] = 0;
                        buffer[offset + 2] = 0;
                        buffer[offset + 3] = 0;
                    }
                }
            })
            .unwrap();

        let mut font = ttf_context.load_font(Path::new(font_file), FONT_SIZE).unwrap();
        font.set_style(STYLE_BOLD);

        let config_root = app_root(AppDataType::UserConfig, &APP_INFO).unwrap();
        let config_file = {
            let mut config_file = config_root.clone();
            config_file.push(CONFIG_FILE);
            config_file
        };

        if !config_file.exists() {
            let default_config = Configuration::default();
            default_config.store(&config_file).unwrap();
        }

        let config = match Configuration::load(&config_file) {
            Err(err) => match err {
                ConfigError::Deserialization(err) => {
                    // Something happend during deserialization, indicating that the file has invalid content
                    println!("config.json could not be deserialized. Replacing with default ({:?})", err);
                    let default_config = Configuration::default();
                    default_config.store(&config_file).unwrap();
                    default_config
                },
                _ => {
                    panic!("Unable to load the configuration file! {:?}", err);
                }
            },
            Ok(config) => {
                config
            }
        };

        Console {
            visible: false,
            visible_start_time: 0,

            config: config,

            font_file: font_file,
            leader: Text::new(ttf_context,
                              &mut renderer,
                              "hakka>",
                              Position::XY(PADDING, height as i32 - FONT_SIZE as i32 - PADDING),
                              FONT_SIZE,
                              FONT_COLOR,
                              font_file),
            input_buffer: "".into(),
            last_command: "".into(),
            command_history: Vec::new(),
            history_position: 0,
            cursor_position: 0,
            buffer: Vec::new(),
            backbuffer_y: 0,
            texture: texture,
            ttf_context: ttf_context,
            size: (width / 2, height),
            font: font,
            input_blocked: false,
        }
    }

    pub fn process(&mut self, event: &Event) {
        // Used to check if no modifiers are held when toggeling console
        let no_mods = |keymod: Mod|
            !keymod.intersects(LALTMOD | LCTRLMOD | LSHIFTMOD | RALTMOD | RCTRLMOD | RSHIFTMOD);
        
        if !self.visible {
            if let Event::KeyDown { scancode, keymod, timestamp, .. } = *event {
                if no_mods(keymod) && scancode == Some(self.config.get_scancode()) {
                    self.toggle(timestamp);
                    return;
                }
            }
            return;
        } 

        // Main event processing, only run if visible
        match *event {
            Event::TextInput { ref text, timestamp, .. } => {
                if self.visible && timestamp > self.visible_start_time + 50 && !self.input_blocked {
                    self.add_text(text);
                }
            }
            Event::MouseWheel { y, .. } => {
                if self.visible &&
                    self.buffer.len() * FONT_SIZE as usize > (self.size.1 - (FONT_SIZE as u32 * 2)) as usize {
                    self.backbuffer_y += y * 6;
                    if self.backbuffer_y < 0 {
                        self.backbuffer_y = 0;
                    }
                }
            }
            Event::KeyDown { keycode, scancode, timestamp, keymod, .. } => {
                if self.visible {
                    if no_mods(keymod) && scancode == Some(self.config.get_scancode()) {
                        self.toggle(timestamp);
                        return;
                    } else if !self.input_blocked {
                        match keycode { 
                            Some(Keycode::C) => {
                                if keymod.intersects(LCTRLMOD | RCTRLMOD) {
                                    self.input_buffer.push_str("^C");
                                    self.commit(false);
                                }
                            }
                            Some(Keycode::Left) => {
                                self.cursor_left();
                            }
                            Some(Keycode::Right) => {
                                self.cursor_right();
                            }
                            Some(Keycode::Backspace) => {
                                self.backspace();
                            }
                            Some(Keycode::Delete) => {
                                if self.cursor_position < self.input_buffer.len() {
                                    self.cursor_position += 1;
                                    self.backspace();
                                }
                            }
                            _ => (),
                        }
                    }
                }
            }
            Event::KeyUp { keycode, timestamp, .. } => {
                if self.visible && !self.input_blocked {
                    match keycode { 
                        Some(Keycode::Up) => {
                            // Special check that an automatic console toggle
                            // does not cause history navigation when holding the
                            // up arrow.
                            if self.visible_start_time > 0 {
                                self.history_navigate_back();
                            } else {
                                self.visible_start_time = timestamp;
                            }
                        }
                        Some(Keycode::Down) => {
                            // Special check that an automatic console toggle
                            // does not cause history navigation when holding the
                            // down arrow.
                            if self.visible_start_time > 0 {
                                self.history_navigate_forward();
                            } else {
                                self.visible_start_time = timestamp;
                            }
                        }
                        Some(Keycode::Return) => {
                            self.commit(true);
                        }
                        Some(Keycode::End) => {
                            self.cursor_position = self.input_buffer.len();
                        }
                        Some(Keycode::Home) => {
                            self.cursor_position = 0;
                        }
                        _ => (),
                    }
                }
            }
            _ => (),
        }
    }

    pub fn process_command(&mut self) {
        let command = self.input_buffer.clone();
        if !command.is_empty() {
            self.command_history.push(command.clone());
            self.last_command = command.clone();
        }
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    fn history_navigate_back(&mut self) {
        if self.history_position > 0 {
            self.input_buffer = self.command_history[self.history_position - 1].clone();
            self.cursor_position = self.input_buffer.len();

            if self.history_position > 0 {
                self.history_position -= 1;
            }
        }
    }

    fn history_navigate_forward(&mut self) {
        if !self.command_history.is_empty() &&
           self.history_position < self.command_history.len() - 1 {
            self.input_buffer = self.command_history[self.history_position + 1].clone();
            self.cursor_position = self.input_buffer.len();
            if self.history_position < self.command_history.len() {
                self.history_position += 1;
            }
        }
    }

    pub fn get_next_command(&mut self) -> Option<String> {
        if !self.last_command.is_empty() {
            let cmd = self.last_command.clone();
            self.last_command.clear();
            Some(cmd)
        } else {
            None
        }
    }

    /// Toggles the visibility of the Console
    pub fn toggle(&mut self, time: u32) {
        self.visible = !self.visible;
        if self.visible {
            self.visible_start_time = time;
        }
    }

    pub fn add_text(&mut self, input: &str) {
        self.input_buffer.insert(self.cursor_position, input.chars().next().unwrap());
        self.cursor_position += input.len();
    }

    pub fn commit(&mut self, execute: bool) {
        let command = self.input_buffer.clone();
        writeln!(self, "hakka> {}", command).unwrap();

        if execute {
            self.process_command();
        }

        self.input_buffer.clear();
        self.cursor_position = 0;
        self.history_position = self.command_history.len();
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
            while !self.input_buffer.is_char_boundary(self.cursor_position) {
                self.cursor_position -= 1;
            }
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
            while !self.input_buffer.is_char_boundary(self.cursor_position) {
                self.cursor_position += 1;
            }
        }
    }

    pub fn backspace(&mut self) {
        if self.visible && self.cursor_position > 0 {
            self.cursor_position -= 1;
            while !self.input_buffer.is_char_boundary(self.cursor_position) {
                self.cursor_position -= 1;
            }
            self.input_buffer.remove(self.cursor_position);
        }
    }

    /// Renders the Console
    pub fn render(&mut self, mut renderer: &mut Renderer) {
        if self.visible {

            renderer.set_blend_mode(BlendMode::Blend);
            self.texture.set_blend_mode(BlendMode::Blend);
            renderer.copy(&self.texture,
                      None,
                      Some(Rect::new(0, 0, self.size.0, self.size.1)))
                .unwrap();
            self.generate_backbuffer_texture(&mut renderer);

            if !self.input_blocked {
                self.render_leader(&mut renderer);
                // Insert the cursor via a dodgy vertical line
                let cursor_x =
                    60 + PADDING as i16 +
                    self.font.size_of(&self.input_buffer[..self.cursor_position]).unwrap().0 as i16;
                // Draw a dodgy cursor
                renderer.thick_line(cursor_x,
                                self.size.1 as i16 - FONT_SIZE as i16 - PADDING as i16,
                                cursor_x,
                                self.size.1 as i16 - PADDING as i16,
                                1,
                                FONT_COLOR)
                    .unwrap();

                if !self.input_buffer.is_empty() {
                    let text = Text::new(self.ttf_context,
                                         &mut renderer,
                                         &self.input_buffer[..],
                                         Position::XY(60 + PADDING,
                                                      self.size.1 as i32 - FONT_SIZE as i32 - PADDING),
                                         FONT_SIZE,
                                         FONT_COLOR,
                                         self.font_file);
                    text.render(&mut renderer);
                }
            } else {
                let text = Text::new(self.ttf_context,
                                     &mut renderer,
                                     "Press Ctrl+C or ENTER to cancel",
                                     Position::XY(PADDING,
                                                  self.size.1 as i32 - FONT_SIZE as i32 - PADDING),
                                     FONT_SIZE,
                                     FONT_COLOR,
                                     self.font_file);
                text.render(&mut renderer);
            }

            self.render_border(&mut renderer);
        }
    }

    fn render_border(&self, mut renderer: &mut Renderer) {
        // Render the border
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 255));
        // North
        renderer.thick_line(0, 0, self.size.0 as i16, 0, 1, BORDER_COLOR).unwrap();

        // East
        renderer.thick_line(self.size.0 as i16,
                        0,
                        self.size.0 as i16,
                        self.size.1 as i16,
                        1,
                        BORDER_COLOR)
            .unwrap();

        // South
        renderer.thick_line(0,
                        self.size.1 as i16 - 1,
                        self.size.0 as i16,
                        self.size.1 as i16 - 1,
                        1,
                        BORDER_COLOR)
            .unwrap();
    }

    fn render_leader(&self, mut renderer: &mut Renderer) {
        // Render a black background behind it so the buffer scrolling looks
        // nicer.
        let rect_y = self.size.1 as i32 - FONT_SIZE as i32 - PADDING;
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        renderer.fill_rect(Rect::new(0, rect_y, self.size.0, rect_y as u32)).unwrap();
        self.leader.render(&mut renderer);
    }

    fn generate_backbuffer_texture(&mut self, mut renderer: &mut Renderer) {
        let mut main_surface = Surface::new(self.size.0,
                                            (self.size.1 - (FONT_SIZE as u32)),
                                            PixelFormatEnum::RGBA8888)
            .unwrap();
        let mut counter = 2;
        // TODO: Make the line render limit here configurable
        for (index, line) in self.buffer.iter().rev().take(200).enumerate() {
            // index 0 is the last line, b/c the iterator is reversed. writeln!
            // outputs a newline at the end of what is written, creating a new
            // string in the buffer, which we do not want to render
            if index == 0 && line.is_empty() {
                continue;
            }

            let y_pos = self.size.1 as i32 - (FONT_SIZE as i32 * counter) + self.backbuffer_y;
            counter += 1;

            if line.trim().is_empty() {
                continue;
            }

            let surface = self.font
                .render(line)
                .blended(FONT_COLOR)
                .unwrap();
            surface.blit(None,
                      &mut main_surface,
                      Some(Rect::new(PADDING, y_pos - PADDING, self.size.1, FONT_SIZE as u32)))
                .unwrap();
        }
        let texture = renderer.create_texture_from_surface(&main_surface)
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();

        renderer.copy(&texture, None, Some(Rect::new(0, 0, width, height))).unwrap();
    }
}

impl<'a> Write for Console<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if self.buffer.is_empty() {
            self.buffer.push(String::new());
        }

        let mut text = String::from(std::str::from_utf8(buf).unwrap());
        while !text.is_empty() {
            if let Some(index) = text.find('\n') {
                let substring: String = text.drain(..index+1).filter(|c| *c != '\n' && *c != '\r').collect();

                if let Some(last) = self.buffer.last_mut() {
                    last.push_str(&substring);
                }

                self.buffer.push(String::new());
            } else {
                if let Some(last) = self.buffer.last_mut() {
                    last.push_str(&text);
                }
                text.drain(..);
            }
        }

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

