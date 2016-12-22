use std;
use std::path::Path;

use sdl2::event::Event;
use sdl2::gfx::primitives::DrawRenderer;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};
use sdl2::render::{BlendMode, Renderer, Texture, TextureQuery};
use sdl2::surface::Surface;
use sdl2::ttf::{Font, Sdl2TtfContext, STYLE_BOLD};

use position::Position;
use text::Text;

const BORDER_COLOR: Color = Color::RGBA(255, 255, 255, 64);

const PADDING: i32 = 10;

const FONT_FILE: &'static str = "../assets/FantasqueSansMono-Bold.ttf";
const FONT_COLOR: Color = Color::RGBA(45, 200, 45, 255);
const FONT_SIZE: u16 = 18;

pub struct Console<'a> {
    pub visible: bool,

    leader: Text,
    input_buffer: String,
    last_command: String,
    command_history: Vec<String>,
    history_position: usize,
    cursor_position: usize,
    buffer: Vec<String>,
    backbuffer_y: i32,
    texture: Texture,
    backbuffer_texture: Texture,
    ttf_context: &'a Sdl2TtfContext,
    size: (u32, u32),
    font: Font<'a>,
}

impl<'a> Console<'a> {
    /// Creates a new empty Console
    pub fn new(ttf_context: &'a Sdl2TtfContext, mut renderer: &mut Renderer) -> Console<'a> {
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
                        buffer[offset + 0] = 182;
                        buffer[offset + 1] = 0;
                        buffer[offset + 2] = 0;
                        buffer[offset + 3] = 0;
                    }
                }
            })
            .unwrap();

        let mut backbuffer_texture =
            renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, width / 2, height)
                .unwrap();
        backbuffer_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..height {
                    for x in 0..width / 2 {
                        let x = x as usize;
                        let y = y as usize;
                        let offset = y * pitch + x * 4;
                        buffer[offset + 0] = 182;
                        buffer[offset + 1] = 0;
                        buffer[offset + 2] = 0;
                        buffer[offset + 3] = 0;
                    }
                }
            })
            .unwrap();

        let mut font = ttf_context.load_font(Path::new(FONT_FILE), FONT_SIZE).unwrap();
        font.set_style(STYLE_BOLD);

        Console {
            visible: false,
            leader: Text::new(&ttf_context,
                              &mut renderer,
                              "hakka>",
                              Position::XY(PADDING, height as i32 - FONT_SIZE as i32 - PADDING),
                              FONT_SIZE,
                              FONT_COLOR,
                              FONT_FILE),
            input_buffer: "".into(),
            last_command: "".into(),
            command_history: Vec::new(),
            history_position: 0,
            cursor_position: 0,
            buffer: Vec::new(),
            backbuffer_y: 0,
            texture: texture,
            backbuffer_texture: backbuffer_texture,
            ttf_context: ttf_context,
            size: (width / 2, height),
            font: font,
        }
    }

    pub fn process(&mut self, event: &Event) {
        match event {
            &Event::KeyUp { keycode: Option::Some(Keycode::Backquote), .. } => {
                self.toggle();
            }
            &Event::TextInput { ref text, .. } => {
                if self.visible {
                    if text == "`" {
                        return;
                    }
                    self.add_text(text);
                }
            }
            &Event::MouseWheel { y, .. } => {
                if self.visible {
                    if self.buffer.len() * FONT_SIZE as usize >
                       (self.size.1 - (FONT_SIZE as u32 * 2)) as usize {
                        self.backbuffer_y += y * 2;
                        if self.backbuffer_y < 0 {
                            self.backbuffer_y = 0;
                        }
                    }
                }
            }
            &Event::KeyUp { keycode, .. } => {
                if self.visible {
                    match keycode { 
                        Some(Keycode::Left) => {
                            self.cursor_left();
                        }
                        Some(Keycode::Right) => {
                            self.cursor_right();
                        }
                        Some(Keycode::Up) => {
                            self.history_navigate_back();
                        }
                        Some(Keycode::Down) => {
                            self.history_navigate_forward();
                        }
                        Some(Keycode::Return) => {
                            self.commit();
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
            _ => (),
        }
    }

    pub fn process_command(&mut self) {
        let command = self.input_buffer.clone();
        self.command_history.push(command.clone());
        self.last_command = command.clone();

        if command == "exit" {
            std::process::exit(0);
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
        if self.history_position < self.command_history.len() - 1 {
            self.input_buffer = self.command_history[self.history_position + 1].clone();
            self.cursor_position = self.input_buffer.len();
            if self.history_position < self.command_history.len() {
                self.history_position += 1;
            }
        }
    }

    pub fn try_process_command(&mut self) -> Option<String> {
        if self.last_command.len() > 0 {
            let cmd = self.last_command.clone();
            self.last_command.clear();
            Some(cmd)
        } else {
            None
        }
    }

    pub fn print<S>(&mut self, text: S)
        where S: Into<String>
    {
        for line in text.into().lines() {
            self.println(line);
        }
    }

    pub fn println<S>(&mut self, text: S)
        where S: Into<String>
    {
        self.buffer.push(text.into());
    }

    /// Toggles the visibility of the Console
    pub fn toggle(&mut self) {
        self.visible = !self.visible;
    }

    pub fn add_text(&mut self, input: &str) {
        self.input_buffer.insert(self.cursor_position, input.chars().next().unwrap());
        self.cursor_position += 1;
    }

    pub fn commit(&mut self) {
        self.buffer.push(format!("hakka> {}", self.input_buffer.clone()));
        self.process_command();
        self.input_buffer.clear();
        self.cursor_position = 0;
        self.history_position = self.command_history.len();
    }

    pub fn cursor_left(&mut self) {
        if self.cursor_position > 0 {
            self.cursor_position -= 1;
        }
    }

    pub fn cursor_right(&mut self) {
        if self.cursor_position < self.input_buffer.len() {
            self.cursor_position += 1;
        }
    }

    pub fn backspace(&mut self) {
        if self.visible {
            if self.cursor_position > 0 {
                self.input_buffer.remove(self.cursor_position - 1);
                self.cursor_position -= 1;
            }
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
            self.render_leader(&mut renderer);
            self.generate_backbuffer_texture(&mut renderer);

            // Insert the cursor via a dodgy vertical line
            let mut output_text = self.input_buffer.clone();
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

            if output_text.len() > 0 {
                let text = Text::new(&self.ttf_context,
                                     &mut renderer,
                                     &output_text[..],
                                     Position::XY(60 + PADDING,
                                                  self.size.1 as i32 - FONT_SIZE as i32 - PADDING),
                                     FONT_SIZE,
                                     FONT_COLOR,
                                     FONT_FILE);
                text.render(&mut renderer);
            }

            self.render_border(&mut renderer);
        }
    }

    fn render_border(&self, mut renderer: &mut Renderer) {
        // Render the border
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 255));
        // North
        renderer.thick_line(0, 0, self.size.0 as i16, 0, 1, BORDER_COLOR);

        // East
        renderer.thick_line(self.size.0 as i16,
                            0,
                            self.size.0 as i16,
                            self.size.1 as i16,
                            1,
                            BORDER_COLOR);

        // South
        renderer.thick_line(0,
                            self.size.1 as i16 - 1,
                            self.size.0 as i16,
                            self.size.1 as i16 - 1,
                            1,
                            BORDER_COLOR);
    }

    fn render_leader(&self, mut renderer: &mut Renderer) {
        self.leader.render(&mut renderer);
    }

    fn generate_backbuffer_texture(&mut self, mut renderer: &mut Renderer) {
        let mut main_surface = Surface::new(self.size.0,
                                            (self.size.1 - (FONT_SIZE as u32)),
                                            PixelFormatEnum::RGBA8888)
            .unwrap();
        let mut counter = 2;
        for line in self.buffer.iter().rev().take(100) {
            let mut y_pos = self.size.1 as i32 - (FONT_SIZE as i32 * counter) + self.backbuffer_y;
            counter += 1;

            if line.trim().len() == 0 {
                continue;
            }

            let surface = self.font
                .render(&line)
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

        renderer.copy(&texture, None, Some(Rect::new(0, 0, width, height)));
    }
}