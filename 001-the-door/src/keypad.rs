
use find_folder::Search;

use sdl2::event::Event;
use sdl2::mouse::MouseButton;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;
use sdl2::ttf::Sdl2TtfContext;

use rs6502::Cpu;

use button::Button;
use lcd::Lcd;

const KEYPAD_COLOR: Color = Color::RGBA(224, 224, 224, 255);

const KEY_WIDTH: u32 = 133;
const KEY_HEIGHT: u32 = 93;

const KEYPAD_X: i32 = 120;
const KEYPAD_Y: i32 = 180;

const KEYPAD_WIDTH: u32 = 500;
const KEYPAD_HEIGHT: u32 = 500;

const KEY_PADDING: u32 = 25;

// ## Hardware registers ##

const KEYPAD_BUTTON_REGISTER: usize = 0xD901;
const KEYPAD_ISR: usize = 0xD100;
const KEYPAD_PWR: usize = 0xD900;

pub struct Keypad {
    lcd: Lcd,
    buttons: Vec<Button>,
}

impl Keypad {
    pub fn new(ttf_context: &Sdl2TtfContext, mut renderer: &mut Renderer) -> Keypad {

        let local = Search::Parents(3).for_folder("001-the-door").unwrap();
        let assets = Search::KidsThenParents(3, 3).for_folder("assets").unwrap();

        let font = assets.join("FantasqueSansMono-Bold.ttf");

        let lcd = Lcd::new(KEYPAD_X + KEY_PADDING as i32,
                           KEYPAD_Y - KEY_HEIGHT as i32,
                           KEYPAD_WIDTH - (KEY_PADDING * 2),
                           KEY_HEIGHT - KEY_PADDING);

        Keypad {
            lcd: lcd,
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

    pub fn process_event(&mut self,
                         event: &Event,
                         ttf_context: &Sdl2TtfContext,
                         mut renderer: &mut Renderer,
                         cpu: &mut Cpu) {

        // If theres no power running to the pin, lets not process any events
        if cpu.memory[KEYPAD_PWR] == 0 {
            return;
        }

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
                                cpu.memory[KEYPAD_BUTTON_REGISTER] = button.value;
                                // Interrupt the CPU to handle this now
                                cpu.memory[KEYPAD_ISR] = 0xFF;
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

    pub fn process(&mut self,
                   ttf_context: &Sdl2TtfContext,
                   mut renderer: &mut Renderer,
                   cpu: &mut Cpu) {
        self.lcd.process(ttf_context, renderer, cpu);
    }

    pub fn render(&mut self, mut renderer: &mut Renderer) {
        renderer.set_draw_color(KEYPAD_COLOR);
        renderer.fill_rect(Rect::new(KEYPAD_X,
                                 self.lcd.rect.top() - KEY_PADDING as i32,
                                 KEYPAD_WIDTH,
                                 self.lcd.rect.height() + KEY_PADDING * 2))
            .unwrap();
        renderer.fill_rect(Rect::new(KEYPAD_X, KEYPAD_Y, KEYPAD_WIDTH, KEYPAD_HEIGHT)).unwrap();
        for button in &self.buttons {
            button.render(renderer);
        }
        self.lcd.render(renderer);
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