use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::Rect;
use sdl2::render::{BlendMode, Renderer, Texture};
use sdl2::ttf::Sdl2TtfContext;

use position::Position;
use text::Text;

pub struct Console<'a> {
    pub visible: bool,

    input_buffer: String,
    cursor_position: usize,
    buffer: Vec<String>,
    texture: Texture,
    ttf_context: &'a Sdl2TtfContext,
}

impl<'a> Console<'a> {
    /// Creates a new empty Console
    pub fn new(ttf_context: &'a Sdl2TtfContext, renderer: &mut Renderer) -> Console<'a> {
        let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGBA8888, 640, 720)
            .unwrap();
        // Create a red-green gradient
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

        Console {
            visible: false,
            input_buffer: "".into(),
            cursor_position: 0,
            buffer: Vec::new(),
            texture: texture,
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
        self.buffer.push(self.input_buffer.clone());
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
            self.input_buffer.remove(self.cursor_position - 1);
            if self.cursor_position > 0 {
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

            let leader = Text::new(&self.ttf_context,
                                   &mut renderer,
                                   "hakka>",
                                   Position::XY(0, 720 - 18),
                                   18,
                                   Color::RGBA(255, 255, 255, 255),
                                   "../assets/FantasqueSansMono-Bold.ttf");
            leader.render(&mut renderer);
            let mut output_text = self.input_buffer.clone();
            if self.cursor_position < output_text.len() {
                output_text.insert(self.cursor_position, '|');
            } else {
                output_text.push_str("|");
            }
            if self.input_buffer.len() > 0 {
                let text = Text::new(&self.ttf_context,
                                     &mut renderer,
                                     &output_text[..],
                                     Position::XY(60, 720 - 18),
                                     18,
                                     Color::RGBA(255, 255, 255, 255),
                                     "../assets/FantasqueSansMono-Bold.ttf");
                text.render(&mut renderer);
            }
        }
    }
}