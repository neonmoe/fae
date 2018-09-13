use png;
use std::error::Error;
use std::fs::File;

pub struct Image {
    pub pixels: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

pub fn load_image(path: &str) -> Result<Image, Box<Error>> {
    let file = File::open(path)?;
    let decoder = png::Decoder::new(file);
    let (info, mut reader) = decoder.read_info()?;
    let mut pixels = vec![0; info.buffer_size()];
    reader.next_frame(&mut pixels)?;
    Ok(Image {
        pixels,
        width: info.width as i32,
        height: info.height as i32,
    })
}
