use std::io::{Read, Seek, SeekFrom};

use anyhow::{ensure, Result};

use crate::jpeg2000::decode;
use crate::tiff::TIFFReader;
use crate::utils::read_le_u32;

mod utils;
mod base;
mod tiff_tags;
mod svs_tags;
mod tiff;
pub mod jpeg2000;


#[derive(Debug)]
pub struct LayerInfo {
    tile_offsets: Vec<u32>,
    tile_bytes: Vec<u32>,
    pub num_tiles_x: u32,
    pub num_tiles_y: u32,
    pub tile_height: u32,
    pub tile_width: u32,
    pub image_height: u32,
    pub image_width: u32,
}

/// Secondary parsing of TIFF headers
#[derive(Debug)]
pub struct SVSHeaders {
    pub layers: Vec<LayerInfo>,
    pub thumbnail: LayerInfo,
}

#[derive(Debug)]
pub struct SVSReader<R: Read + Seek> {
    tiffreader: TIFFReader<R>,
    pub headers: SVSHeaders,
}

impl<R: Read + Seek> SVSReader<R> {
    pub fn open(reader: R) -> Result<Self> {
        let mut tiffreader = TIFFReader::open(reader)?;

        let headers = Self::read_headers(&mut tiffreader)?;

        Ok(Self {
            tiffreader,
            headers,
        })
    }

    fn read_headers(tiffreader: &mut TIFFReader<R>) -> Result<SVSHeaders> {
        let mut layers: Vec<LayerInfo> = vec![];
        let mut thumbnail: Option<LayerInfo> = None;


        for i in 0..tiffreader.headers.directories.len() {
            if i == 1 {
                /* Thumbnail */

                let tile_offsets = tiffreader.get_tag_value(i, 273)?
                    .chunks(4)
                    .map(read_le_u32)
                    .collect::<Vec<_>>();
                let tile_bytes = tiffreader.get_tag_value(i, 279)?
                    .chunks(4)
                    .map(read_le_u32)
                    .collect::<Vec<_>>();

                thumbnail = Some(LayerInfo {
                    tile_offsets,
                    tile_bytes,
                    num_tiles_x: 0, // todo
                    num_tiles_y: 0,
                    tile_height: 0,
                    tile_width: 0,
                    image_height: 0,
                    image_width: 0,
                })
            } else {
                /* Tiled layer */

                let tile_offsets = tiffreader.get_tag_value(i, 324)?
                    .chunks(4)
                    .map(read_le_u32)
                    .collect::<Vec<_>>();
                let tile_bytes = tiffreader.get_tag_value(i, 325)?
                    .chunks(4)
                    .map(read_le_u32)
                    .collect::<Vec<_>>();

                let tile_width = tiffreader.get_tag_value(i, 322)?;
                let tile_width = read_le_u32(&tile_width);
                let tile_height = tiffreader.get_tag_value(i, 323)?;
                let tile_height = read_le_u32(&tile_height);

                let image_width = tiffreader.get_tag_value(i, 256)?;
                let image_width = read_le_u32(&image_width);
                let image_height = tiffreader.get_tag_value(i, 257)?;
                let image_height = read_le_u32(&image_height);

                let num_tiles_x = image_width.div_ceil(tile_width);
                let num_tiles_y = image_height.div_ceil(tile_height);

                layers.push(LayerInfo {
                    tile_offsets,
                    tile_bytes,
                    num_tiles_x,
                    num_tiles_y,
                    tile_height,
                    tile_width,
                    image_height,
                    image_width,
                });
            }
        }

        Ok(SVSHeaders {
            layers,
            thumbnail: thumbnail.unwrap(),
        })
    }

    /// Reads uncompressed tile data
    pub fn read_tile_compressed(&mut self, layer: usize, tile_id: usize) -> Result<Vec<u8>> {
        ensure!(layer < self.headers.layers.len(), "Invalid layer given.");

        // Get layer
        let layer = &self.headers.layers[layer];
        ensure!(tile_id < layer.tile_bytes.len(), "Invalid tile id given. {:?} {}", layer, layer.tile_bytes.len());

        let offset = layer.tile_offsets[tile_id];
        let bytes = layer.tile_bytes[tile_id];

        // Seek & read
        let mut buf = vec![0; bytes as usize];
        self.tiffreader.reader.seek(SeekFrom::Start(offset as u64))?;
        self.tiffreader.reader.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn read_tile_uncompressed(&mut self, layer: usize, tile_id: usize) -> Result<Vec<u8>> {
        decode(&self.read_tile_compressed(layer, tile_id)?)
    }

    pub fn layer_scale(&self, layer: usize) -> Result<f32> {
        ensure!(layer < self.headers.layers.len(), "Invalid layer given.");

        Ok(self.headers.layers[layer].image_width as f32 / self.headers.layers[0].image_width as f32)
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;

    use rayon::iter::ParallelIterator;
    use rayon::prelude::ParallelBridge;

    use crate::jpeg2000::decode;
    use crate::SVSReader;

    #[test]
    fn test() {
        let source_file = r"E:\datasets\tcga\images\hcmi_cmdc\0a5410d7-0f5c-4dda-986e-d857d176498f\HCM-CSHL-0366-C50-01A-01-S1-HE.30E0E448-BC32-4FCA-99F8-CF5E8C283352.svs";
        let file = File::open(source_file).expect("Failed to open file");
        let mut svs = SVSReader::open(file).expect("Failed to read svs.");
        // println!("{:?}", svs);


        let mut buf = vec![0_u8; 240 * 240 * 3];


        let layer = 0;
        let tiles = (0..svs.headers.layers[layer].tile_offsets.len())
            .map(|i| svs.read_tile_compressed(layer, i).unwrap())
            .par_bridge()
            .map(|b| decode(&b).unwrap())
            .count();
        println!("{}", tiles);
    }
}