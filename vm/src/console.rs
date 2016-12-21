use std::path::Path;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Renderer, Texture, TextureQuery};
use sdl2::surface::Surface;
use sdl2::ttf::{Sdl2TtfContext, STYLE_BOLD};

use position::Position;
use text::Text;

const FONT_FILE: &'static str = "../assets/FantasqueSansMono-Bold.ttf";
const FONT_COLOR: Color = Color::RGBA(45, 200, 45, 255);

pub struct Console<'a> {
    pub visible: bool,

    leader: Text,
    input_buffer: String,
    cursor_position: usize,
    buffer: Vec<String>,
    texture: Texture,
    backbuffer_texture: Texture,
    ttf_context: &'a Sdl2TtfContext,
}

impl<'a> Console<'a> {
    /// Creates a new empty Console
    pub fn new(ttf_context: &'a Sdl2TtfContext, mut renderer: &mut Renderer) -> Console<'a> {
        let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, 640, 720)
            .unwrap();
        texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..720 {
                    for x in 0..640 {
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
            renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, 640, 720)
                .unwrap();
        backbuffer_texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
                for y in 0..720 {
                    for x in 0..640 {
                        let offset = y * pitch + x * 4;
                        buffer[offset + 0] = 182;
                        buffer[offset + 1] = 0;
                        buffer[offset + 2] = 0;
                        buffer[offset + 3] = 0;
                    }
                }
            })
            .unwrap();

        Console {
            visible: false,
            leader: Text::new(&ttf_context,
                              &mut renderer,
                              "hakka>",
                              Position::XY(0, 720 - 18),
                              18,
                              FONT_COLOR,
                              FONT_FILE),
            input_buffer: "".into(),
            cursor_position: 0,
            buffer: Vec::new(),
            texture: texture,
            backbuffer_texture: backbuffer_texture,
            ttf_context: ttf_context,
        }
    }

    pub fn process(&mut self, event: &Event) {
        match event {
            &Event::TextInput { ref text, .. } => {
                if self.visible {
                    if text == "`" {
                        return;
                    }
                    self.add_text(text);
                }
            }
            &Event::KeyUp { keycode: Option::Some(Keycode::Backquote), .. } => {
                self.toggle();
            }
            &Event::KeyUp { keycode: Option::Some(Keycode::Left), .. } => {
                if self.visible {
                    self.cursor_left();
                }
            }
            &Event::KeyUp { keycode: Option::Some(Keycode::Right), .. } => {
                if self.visible {
                    self.cursor_right();
                }
            }
            &Event::KeyUp { keycode: Option::Some(Keycode::Return), .. } => {
                if self.visible {
                    self.commit();
                }
            }
            &Event::KeyDown { keycode: Option::Some(Keycode::Backspace), .. } => {
                if self.visible {
                    self.backspace();
                }
            }
            _ => (),
        }
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
        self.input_buffer.clear();
        self.cursor_position = 0;
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
            renderer.copy(&self.texture, None, Some(Rect::new(0, 0, 640, 720))).unwrap();
            self.render_leader(&mut renderer);
            self.generate_backbuffer_texture(&mut renderer);
            // self.render_buffer(&mut renderer);

            // Insert the cursor in its proper position
            let mut output_text = self.input_buffer.clone();
            if self.cursor_position < output_text.len() {
                output_text.insert(self.cursor_position, '|');
            } else {
                output_text.push_str("|");
            }

            // If no input.. only draw the cursor
            if self.input_buffer.len() == 0 {
                output_text = "|".into();
            }

            let text = Text::new(&self.ttf_context,
                                 &mut renderer,
                                 &output_text[..],
                                 Position::XY(60, 720 - 18),
                                 18,
                                 FONT_COLOR,
                                 FONT_FILE);
            text.render(&mut renderer);
        }
    }

    fn render_leader(&self, mut renderer: &mut Renderer) {
        self.leader.render(&mut renderer);
    }

    fn generate_backbuffer_texture(&mut self, mut renderer: &mut Renderer) {
        let mut font = self.ttf_context.load_font(Path::new(FONT_FILE), 18).unwrap();
        font.set_style(STYLE_BOLD);
        let mut main_surface = Surface::new(640, 702, PixelFormatEnum::RGBA8888).unwrap();
        let mut counter = 2;
        for line in self.buffer.iter().rev() {
            let surface = font.render(&line)
                .blended(FONT_COLOR)
                .unwrap();
            surface.blit(None,
                      &mut main_surface,
                      Some(Rect::new(0, 720 - (18 * counter), 720, 18)))
                .unwrap();
            counter += 1;
        }
        let texture = renderer.create_texture_from_surface(&main_surface)
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();

        renderer.copy(&texture, None, Some(Rect::new(0, 0, width, height)));
    }
}