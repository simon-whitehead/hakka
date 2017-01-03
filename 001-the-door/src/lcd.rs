use find_folder::Search;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;

use rs6502::Cpu;

use vm::{Position, Text};

const LCD_COLOR: Color = Color::RGBA(192, 192, 192, 255);
const LCD_BACKCOLOR: Color = Color::RGBA(0, 0, 255, 255);
const LCD_ISR: usize = 0xD101;
const LCD_PWR: usize = 0xD800;
const LCD_CTRL_REGISTER: usize = 0xD801;

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
            color: LCD_COLOR,
            power: false,
        }
    }

    pub fn process(&mut self,
                   ttf_context: &Sdl2TtfContext,
                   mut renderer: &mut Renderer,
                   cpu: &mut Cpu,
                   addr: u16) {
        let addr = addr as usize;
        match Self::get_mode(cpu) {
            LcdMode::Text => {
                // Clear the buffer
                self.buffer.clear();
                // Read each character from the CPU memory and store it in our buffer
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
                    let mut text_object =
                        Text::new(ttf_context,
                                  renderer,
                                  self.buffer.clone(),
                                  Position::HorizontalCenter(self.rect.left() +
                                                             (self.rect.width() as i32 / 2),
                                                             self.rect.top() + 10),
                                  60,
                                  self.color,
                                  self.font.clone());

                    self.text = Some(text_object);
                } else {
                    self.text = None
                };
            }
            LcdMode::Clear => {
                cpu.memory[LCD_ISR] = 0xFF;
                cpu.irq();
            }
            LcdMode::SetColor => {
                let r = cpu.memory[addr];
                let g = cpu.memory[addr + 0x01];
                let b = cpu.memory[addr + 0x02];

                self.color = Color::RGBA(r, g, b, 255);
            }
            LcdMode::Unknown => (),
        }

        self.power = cpu.memory[LCD_PWR] != 0
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

        renderer.set_draw_color(LCD_BACKCOLOR);
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