
use find_folder::Search;

use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;

use rs6502::Cpu;

use button::Button;

const KEYPAD_COLOR: Color = Color::RGBA(127, 127, 127, 255);

const KEY_WIDTH: u32 = 133;
const KEY_HEIGHT: u32 = 93;

const KEYPAD_X: i32 = 120;
const KEYPAD_Y: i32 = 120;

const KEYPAD_WIDTH: u32 = 500;
const KEYPAD_HEIGHT: u32 = 500;

const KEY_PADDING: u32 = 25;

// ## Hardware registers ##

const HARDWARE_REG_BUTTON: usize = 0xD0FF;

pub struct Keypad {
    buttons: Vec<Button>,
}

impl Keypad {
    pub fn new(ttf_context: &Sdl2TtfContext, mut renderer: &mut Renderer) -> Keypad {

        let local = Search::Parents(3).for_folder("001-the-door").unwrap();
        let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

        let font = assets.join("FantasqueSansMono-Bold.ttf");

        Keypad {
            buttons: vec![
                Button::new(ttf_context, renderer, "1", None, 1, Self::create_button_rect(0, 0), font.clone()),
                 Button::new(ttf_context, renderer, "2", "ABC", 2, Self::create_button_rect(1, 0), font.clone()),
                 Button::new(ttf_context, renderer, "3", "DEF", 3, Self::create_button_rect(2, 0), font.clone()),
                 Button::new(ttf_context, renderer, "4", "GHI", 4, Self::create_button_rect(0, 1), font.clone()),
                 Button::new(ttf_context, renderer, "5", "JKL", 5, Self::create_button_rect(1, 1), font.clone()),
                 Button::new(ttf_context, renderer, "6", "MNO", 6, Self::create_button_rect(2, 1), font.clone()),
                  Button::new(ttf_context, renderer, "7", "PQRS", 7, Self::create_button_rect(0, 2), font.clone()),
                 Button::new(ttf_context, renderer, "8", "TUV", 8, Self::create_button_rect(1, 2), font.clone()),
                  Button::new(ttf_context, renderer, "9", "WXYZ", 9, Self::create_button_rect(2, 2), font.clone()),
                Button::new(ttf_context, renderer, "*", None, 254, Self::create_button_rect(0, 3), font.clone()),
                Button::new(ttf_context, renderer, "0", None, 0, Self::create_button_rect(1, 3), font.clone()),
                Button::new(ttf_context, renderer, "#", None, 255, Self::create_button_rect(2, 3), font.clone()),
            ],
        }
    }

    pub fn process(&mut self, event: &Event, cpu: &mut Cpu) {
        match *event {
            Event::MouseButtonDown { mouse_btn, x, y, .. } => {
                if mouse_btn == MouseButton::Left {
                    if (x > KEYPAD_X && x < KEYPAD_X + KEYPAD_WIDTH as i32) &&
                       (y > KEYPAD_Y && y < KEYPAD_Y + KEYPAD_HEIGHT as i32) {
                        // We have clicked inside the keypad - lets see if we hit a button:
                        for button in &self.buttons {
                            if (x > button.rect.left() && x < button.rect.right()) &&
                               (y > button.rect.top() && y < button.rect.bottom()) {
                                // Yep, we clicked this button, put its value in the hardware register
                                cpu.memory[HARDWARE_REG_BUTTON] = button.value;
                                // Interrupt the CPU to handle this now
                                cpu.irq();
                                break;
                            }
                        }
                    }
                }
            }
            _ => (),
        }
    }

    pub fn render(&self, mut renderer: &mut Renderer) {
        renderer.set_draw_color(KEYPAD_COLOR);
        renderer.fill_rect(Rect::new(KEYPAD_X, KEYPAD_Y, KEYPAD_WIDTH, KEYPAD_HEIGHT)).unwrap();
        for button in &self.buttons {
            button.render(renderer);
        }
    }

    fn create_button_rect(x: u32, y: u32) -> Rect {
        let button_x = KEY_PADDING * (x + 1) + (x * KEY_WIDTH);
        let button_y = KEY_PADDING * (y + 1) + (y * KEY_HEIGHT);

        Rect::new(KEYPAD_X + button_x as i32,
                  KEYPAD_Y + button_y as i32,
                  KEY_WIDTH,
                  KEY_HEIGHT)
    }
}