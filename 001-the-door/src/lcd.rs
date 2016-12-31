use find_folder::Search;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;

use rs6502::Cpu;

use vm::{Position, Text};

pub struct Lcd {
    x: i32,
    y: i32,
    font: String,
    buffer: String,
    text: Option<Text>,
}

impl Lcd {
    pub fn new(x: i32, y: i32) -> Lcd {

        let local = Search::Parents(3).for_folder("001-the-door").unwrap();
        let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

        let font = assets.join("Segment7Standard.otf");

        Lcd {
            x: x,
            y: y,
            font: font.to_str().unwrap().into(),
            text: None,
            buffer: "".into(),
        }
    }

    pub fn process(&mut self,
                   ttf_context: &Sdl2TtfContext,
                   mut renderer: &mut Renderer,
                   cpu: &mut Cpu,
                   addr: u16) {
        let addr = addr as usize;
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
            self.text = Some(Text::new(ttf_context,
                                       renderer,
                                       self.buffer.clone(),
                                       Position::XY(self.x, self.y),
                                       72,
                                       Color::RGBA(0, 255, 255, 255),
                                       self.font.clone()));
        }
    }

    pub fn render(&mut self, mut renderer: &mut Renderer) {
        if let Some(ref text) = self.text {
            text.render(renderer);
        }
    }
}