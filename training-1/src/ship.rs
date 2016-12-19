use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};

pub struct Ship {
    pub x: i32,
    pub y: i32,
    width: u32,
    height: u32,
    flame_width: u32,
    flame_height: u32,
    texture: Texture,
    flame_texture: Texture,
}

impl Ship {
    pub fn new(texture: Texture, flame_texture: Texture) -> Ship {
        let TextureQuery { width: ship_width, height: ship_height, .. } = texture.query();
        let TextureQuery { width: flame_width, height: flame_height, .. } = flame_texture.query();

        Ship {
            x: 0,
            y: 1200,
            width: ship_width,
            height: ship_height,
            texture: texture,
            flame_width: flame_width,
            flame_height: flame_height,
            flame_texture: flame_texture,
        }
    }

    pub fn process(&mut self, memory: &[u8]) {
        // println!("Mem: {:?}", &memory[0x00..0x05]);
        let mut x: u16 = memory[0x00] as u16;
        x |= (memory[0x01] as u16) << 8;
        self.x = x as i32;

        let mut y: u16 = memory[0x02] as u16;
        y |= (memory[0x03] as u16) << 8;
        self.y = y as i32;
    }

    pub fn render(&self, renderer: &mut Renderer) {
        renderer.set_draw_color(Color::RGBA(0, 0, 0, 255));
        renderer.copy(&self.texture,
                  None,
                  Some(Rect::new(self.x, self.y, self.width, self.height)))
            .unwrap();
    }

    pub fn render_flame(&self, renderer: &mut Renderer) {
        renderer.copy(&self.flame_texture,
                  None,
                  Some(Rect::new(self.x - 10,
                                 self.y + 150,
                                 self.flame_width,
                                 self.flame_height)))
            .unwrap();

        renderer.copy(&self.flame_texture,
                  None,
                  Some(Rect::new(self.x + 77,
                                 self.y + 150,
                                 self.flame_width,
                                 self.flame_height)))
            .unwrap();
    }
}