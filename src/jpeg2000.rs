use anyhow::Result;
use jpeg2k::{DecodeParameters, Image};

pub fn decode_into_buffer(bytes: &[u8], buf: &mut [u8]) -> Result<()> {
    let decode_params = DecodeParameters::default();

    let jp2_image = Image::from_bytes_with(bytes, decode_params)?;

    assert!(
        buf.len() >= (jp2_image.height() * jp2_image.width() * jp2_image.num_components()) as usize
    );

    let components = jp2_image.components();
    for i in 0..components.len() {
        let data = components[i].data();
        for j in 0..data.len() {
            buf[j * 3 + i] = data[j] as u8;
        }
    }

    Ok(())
}

pub fn decode(bytes: &[u8]) -> Result<Vec<u8>> {
    let decode_params = DecodeParameters::default()
        // .decode_area(Some(DecodeArea::new(0, 0, 1, 1)))
        ;

    let jp2_image = Image::from_bytes_with(bytes, decode_params)?;

    let mut buf =
        vec![0; (jp2_image.height() * jp2_image.width() * jp2_image.num_components()) as usize];

    let components = jp2_image.components();
    for i in 0..components.len() {
        let data = components[i].data();
        for j in 0..data.len() {
            buf[j * 3 + i] = data[j] as u8;
        }
    }

    Ok(buf)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Read;

    use crate::jpeg2000::{decode, decode_into_buffer};

    #[test]
    fn test_with_buffer() {
        let bytes = fs::read("1.j2k").unwrap();

        let mut big_buf = vec![0_u8; 240 * 240 * 3];
        for _ in 0..1000 {
            decode_into_buffer(&bytes, &mut big_buf).unwrap();
        }
        println!("{}", big_buf[big_buf.len() - 1]);
    }

    #[test]
    fn test_decode() {
        let bytes = fs::read("1.j2k").unwrap();

        for _ in 0..1000 {
            let buffer = decode(&bytes).unwrap();
        }
    }
}

