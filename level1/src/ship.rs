use sdl2::rect::Rect;
use sdl2::render::{Renderer, Texture, TextureQuery};

pub struct Ship {
    x: i32,
    y: i32,
    width: u32,
    height: u32,
    texture: Texture,
}

impl Ship {
    pub fn new(texture: Texture) -> Ship {
        let TextureQuery { width, height, .. } = texture.query();

        Ship {
            x: 0,
            y: 0,
            width: width,
            height: height,
            texture: texture,
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
        renderer.copy(&self.texture,
                      None,
                      Some(Rect::new(self.x, self.y, self.width, self.height)));
    }
}