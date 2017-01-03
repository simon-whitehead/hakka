use byteorder::{ByteOrder, LittleEndian};

use find_folder::Search;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;

use rs6502::Cpu;

use vm::{Position, Text};

const LCD_DISPLAY_INPUT_BUFFER: usize = 0xD010;
const LCD_DISPLAY_BUFFER_POINTER: usize = 0xD006;

const LCD_COLOR: usize = 0xD000;
const LCD_BACKCOLOR: usize = 0xD003;

const LCD_ISR: usize = 0xD101;
const LCD_PWR: usize = 0xD800;
const LCD_CTRL_REGISTER: usize = 0xD801;

const LCD_FONT_SIZE: u16 = 48;

enum LcdMode {
    Text,
    Clear,
    SetColor,
    Unknown,
}

pub struct Lcd {
    pub rect: Rect,
    font: String,
    buffer: String,
    text: Option<Text>,
    mode: LcdMode,
    color: Color,
    back_color: Color,
    power: bool,
}

impl Lcd {
    pub fn new(x: i32, y: i32, width: u32, height: u32) -> Lcd {

        let local = Search::Parents(3).for_folder("001-the-door").unwrap();
        let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

        let font = assets.join("Segment7Standard.otf");

        Lcd {
            rect: Rect::new(x, y, width, height),
            font: font.to_str().unwrap().into(),
            text: None,
            buffer: "".into(),
            mode: LcdMode::Text,
            color: Color::RGBA(255, 255, 255, 255),
            back_color: Color::RGBA(0, 0, 0, 255),
            power: false,
        }
    }

    pub fn process(&mut self,
                   ttf_context: &Sdl2TtfContext,
                   mut renderer: &mut Renderer,
                   cpu: &mut Cpu) {
        match Self::get_mode(cpu) {
            LcdMode::Text => {
                // Clear the buffer
                self.buffer.clear();
                // Read each character from the CPU memory and store it in our buffer
                let addr =
                    LittleEndian::read_u16(&cpu.memory[LCD_DISPLAY_BUFFER_POINTER..]) as usize;
                for byte in &cpu.memory[addr..] {
                    // If its a null terminator (lol... see whats happening here?) break out
                    if *byte == 0x00 {
                        break;
                    }

                    // Otherwise, convert it to a character and append it to the buffer
                    self.buffer.push(*byte as char);
                }

                // Create a text object if we have some text
                if !self.buffer.is_empty() {
                    self.generate_text(ttf_context, renderer);
                } else {
                    self.text = None
                };
            }
            LcdMode::Clear => {
                cpu.memory[LCD_ISR] = 0xFF;
                cpu.irq();
            }
            LcdMode::SetColor => {
                let r = cpu.memory[LCD_COLOR];
                let g = cpu.memory[LCD_COLOR + 0x01];
                let b = cpu.memory[LCD_COLOR + 0x02];

                let new_color = Color::RGBA(r, g, b, 255);
                if new_color != self.color {
                    self.color = new_color;
                    self.generate_text(ttf_context, renderer);
                }

                let r = cpu.memory[LCD_BACKCOLOR];
                let g = cpu.memory[LCD_BACKCOLOR + 0x01];
                let b = cpu.memory[LCD_BACKCOLOR + 0x02];

                self.back_color = Color::RGBA(r, g, b, 255);
            }
            LcdMode::Unknown => (),
        }

        self.power = cpu.memory[LCD_PWR] != 0
    }

    fn generate_text(&mut self, ttf_context: &Sdl2TtfContext, renderer: &mut Renderer) {
        if self.buffer.is_empty() {
            self.text = None;
        } else {
            let mut text_object = Text::new(ttf_context,
                                            renderer,
                                            self.buffer.clone(),
                                            Position::HorizontalCenter(self.rect.left() +
                                                                       (self.rect.width() as i32 /
                                                                        2),
                                                                       self.rect.top() + 10),
                                            LCD_FONT_SIZE,
                                            self.color,
                                            self.font.clone());

            self.text = Some(text_object);
        }
    }

    fn get_mode(cpu: &mut Cpu) -> LcdMode {
        match *&cpu.memory[LCD_CTRL_REGISTER] {
            0 => LcdMode::Text,
            1 => LcdMode::Clear,
            2 => LcdMode::SetColor,
            _ => LcdMode::Unknown,
        }
    }

    pub fn render(&mut self, mut renderer: &mut Renderer) {
        if !self.power {
            return;
        }

        renderer.set_draw_color(self.back_color);
        renderer.fill_rect(Rect::new(self.rect.left(),
                                     self.rect.top(),
                                     self.rect.width(),
                                     self.rect.height()));

        if let Some(ref mut text) = self.text {
            renderer.set_draw_color(self.color);
            text.render(renderer);
        }
    }
}