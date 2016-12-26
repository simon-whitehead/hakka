use std::path::Path;

use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};
use sdl2::ttf::{Sdl2TtfContext, STYLE_BOLD};

use position::Position;

pub struct Text {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    texture: Texture,
}

impl Text {
    pub fn new<S, P>(ttf_context: &Sdl2TtfContext,
                     renderer: &mut Renderer,
                     text: S,
                     position: Position,
                     font_size: u16,
                     color: Color,
                     path: P)
                     -> Text
        where S: Into<String>,
              P: AsRef<Path>
    {
        let mut font = ttf_context.load_font(path.as_ref(), font_size).unwrap();
        font.set_style(STYLE_BOLD);
        let surface = font.render(&text.into())
            .blended(color)
            .unwrap();
        let texture = renderer.create_texture_from_surface(&surface)
            .unwrap();

        let TextureQuery { width, height, .. } = texture.query();

        let (x, y) = match position {
            Position::HorizontalCenter(x, y) => (x - width as i32 / 2, y),
            Position::XY(x, y) => (x, y),
        };

        Text {
            x: x,
            y: y,
            width: width,
            height: height,
            texture: texture,
        }
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.copy(&self.texture,
                  None,
                  Some(Rect::new(self.x, self.y, self.width, self.height)))
            .unwrap();
    }
}