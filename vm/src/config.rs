
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
        let encoded = json::as_pretty_json(&self);
        let mut file = File::create(target)?;
        write!(file, "{}", encoded)?;
        writeln!(file, "")?; // End file with newline
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
    File(io::Error),
    Serialization(json::EncoderError),
    Deserialization(json::DecoderError),
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> ConfigError {
        ConfigError::File(e) 
    }
}
impl From<json::EncoderError> for ConfigError {
    fn from(e: json::EncoderError) -> ConfigError {
        ConfigError::Serialization(e) 
    }
}
impl From<json::DecoderError> for ConfigError {
    fn from(e: json::DecoderError) -> ConfigError {
        ConfigError::Deserialization(e) 
    }
}


