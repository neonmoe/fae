use png;
use std::error::Error;
use std::io::Read;

pub struct Image {
    pub pixels: Vec<u8>,
    pub width: i32,
    pub height: i32,
}

pub fn load_image<R: Read>(bytes: R) -> Result<Image, Box<Error>> {
    let decoder = png::Decoder::new(bytes);
    let (info, mut reader) = decoder.read_info()?;
    let mut pixels = vec![0; info.buffer_size()];
    reader.next_frame(&mut pixels)?;
    Ok(Image {
        pixels,
        width: info.width as i32,
        height: info.height as i32,
    })
}
