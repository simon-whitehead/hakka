
use sdl2::keyboard::Scancode;
use rustc_serialize::json;
use std::path::Path;
use std::fs::File;
use std::io::{Read, Write};
use std::io;

const DEFAULT_CONSOLE_TOGGLE: Scancode = Scancode::Grave;

#[derive(RustcDecodable, RustcEncodable, Debug)]
pub struct Configuration {
    console_toggle: i32,
}

impl Configuration {
    pub fn default() -> Configuration {
        Configuration {
            console_toggle: DEFAULT_CONSOLE_TOGGLE as i32
        }
    }

    pub fn store(&self, target: &Path) -> Result<(), ConfigError> {
        let encoded = json::encode(&self)?;

        let mut file = File::create(target)?;
        write!(file, "{}", encoded)?;

        Ok(())
    }

    pub fn load(target: &Path) -> Result<Configuration, ConfigError> {
        let mut file = File::open(target)?;
        let mut buffer = String::new();
        file.read_to_string(&mut buffer)?;
        let config = json::decode(&buffer)?;
        Ok(config)
    }

    pub fn get_scancode(&self) -> Scancode {
        Scancode::from_i32(self.console_toggle).unwrap_or(DEFAULT_CONSOLE_TOGGLE)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    FileError(io::Error),
    SerializationError(json::EncoderError),
    DeserializationError(json::DecoderError),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        ConfigError::FileError(e) 
    }
}
impl From<json::EncoderError> for ConfigError {
    fn from(e: json::EncoderError) -> ConfigError {
        ConfigError::SerializationError(e) 
    }
}
impl From<json::DecoderError> for ConfigError {
    fn from(e: json::DecoderError) -> ConfigError {
        ConfigError::DeserializationError(e) 
    }
}


