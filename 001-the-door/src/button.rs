use std::path::Path;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Renderer;

use sdl2::ttf::Sdl2TtfContext;

use vm::{Position, Text};

const FONT_SIZE: u16 = 60;
const SUBTEXT_FONT_SIZE: u16 = 28;

pub struct Button {
    text: String,
    subtext: String,
    rect: Rect,
    text_object: Text,
    subtext_object: Option<Text>,
}

impl Button {
    pub fn new<S, OS, P>(ttf_context: &Sdl2TtfContext,
                         mut renderer: &mut Renderer,
                         text: S,
                         subtext: OS,
                         rect: Rect,
                         font: P)
                         -> Button
        where S: Into<String> + Default,
              OS: Into<Option<S>>,
              P: AsRef<Path> + Clone
    {

        let text = text.into();
        let text_position = Position::HorizontalCenter(rect.left() + (rect.width() / 2) as i32,
                                                       rect.top());

        let subtext = subtext.into().unwrap_or(Default::default()).into();
        let subtext_position = Position::HorizontalCenter(rect.left() + (rect.width() / 2) as i32,
                                                          rect.bottom() - SUBTEXT_FONT_SIZE as i32 -
                                                          10);

        let text_object = Self::create_text_object(ttf_context,
                                                   renderer,
                                                   text.clone(),
                                                   &rect,
                                                   font.clone(),
                                                   FONT_SIZE,
                                                   text_position);

        let subtext_object = if subtext.len() == 0 {
            None
        } else {
            Some(Self::create_text_object(ttf_context,
                                          renderer,
                                          subtext.clone(),
                                          &rect,
                                          font.clone(),
                                          SUBTEXT_FONT_SIZE,
                                          subtext_position))
        };

        Button {
            text: text.clone(),
            subtext: subtext.clone(),
            rect: rect,
            text_object: text_object,
            subtext_object: subtext_object,
        }
    }

    pub fn render(&self, mut renderer: &mut Renderer) {
        renderer.set_draw_color(Color::RGBA(255, 255, 255, 255));
        renderer.fill_rect(self.rect);
        self.text_object.render(renderer);
        if let Some(ref subtext) = self.subtext_object {
            subtext.render(renderer);
        }
    }

    fn create_text_object<P>(ttf_context: &Sdl2TtfContext,
                             mut renderer: &mut Renderer,
                             text: String,
                             rect: &Rect,
                             font: P,
                             font_size: u16,
                             position: Position)
                             -> Text
        where P: AsRef<Path>
    {
        Text::new(ttf_context,
                  renderer,
                  text,
                  position,
                  font_size,
                  Color::RGBA(0, 0, 0, 255),
                  font)
    }
}