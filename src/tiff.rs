use std::io::{Read, Seek, SeekFrom};

use anyhow::{bail, ensure, Result};

use crate::base::Endianness;
use crate::utils::{read_le_u16, read_le_u32};

#[derive(Debug, Clone)]
pub enum FieldType {
    Byte,
    Ascii,
    Short,
    Long,
    Rational,
    SByte,
    Undefined,
    SShort,
    Slong,
    SRational,
    Float,
    Double,
}

impl FieldType {
    fn from_u16(x: u16) -> anyhow::Result<Self> {
        let matched = match x {
            1 => Self::Byte,
            2 => Self::Ascii,
            3 => Self::Short,
            4 => Self::Long,
            5 => Self::Rational,
            6 => Self::SByte,
            7 => Self::Undefined,
            8 => Self::SShort,
            9 => Self::Slong,
            10 => Self::SRational,
            11 => Self::Float,
            12 => Self::Double,
            _ => bail!("Invalid FieldType: {x}")
        };

        Ok(matched)
    }

    fn num_bytes(&self) -> u8 {
        match self {
            FieldType::Byte => 1,
            FieldType::Ascii => 1,
            FieldType::Short => 2,
            FieldType::Long => 4,
            FieldType::Rational => 8,
            FieldType::SByte => 1,
            FieldType::Undefined => 1,
            FieldType::SShort => 2,
            FieldType::Slong => 4,
            FieldType::SRational => 8,
            FieldType::Float => 4,
            FieldType::Double => 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct IDFEntry {
    tag: u16,
    field_type: FieldType,
    count: u32,
    offset: u32,
}

impl IDFEntry {
    pub fn from_buf(buf: &[u8]) -> anyhow::Result<Self> {
        assert_eq!(buf.len(), 12);

        Ok(Self {
            tag: read_le_u16(&buf[..2]),
            field_type: FieldType::from_u16(read_le_u16(&buf[2..4]))?,
            count: read_le_u32(&buf[4..8]),
            offset: read_le_u32(&buf[8..12]),
        })
    }
}

#[derive(Debug)]
pub struct IDFDirectory {
    offset: u32,
    entries: Vec<IDFEntry>,
}

impl IDFDirectory {
    pub fn get_entry_by_tag_id(&self, id: u16) -> anyhow::Result<&IDFEntry> {
        match self.entries.iter().find(|e| e.tag == id) {
            None => bail!("Tag not found: {id}"),
            Some(e) => Ok(e)
        }
    }
}

#[derive(Debug)]
pub struct TIFFHeaders {
    endianness: Endianness,
    pub directories: Vec<IDFDirectory>,
}

#[derive(Debug)]
pub struct TIFFReader<R: Read + Seek> {
    pub reader: R,
    pub headers: TIFFHeaders,
}


impl<R: Read + Seek> TIFFReader<R> {
    pub fn open(mut reader: R) -> anyhow::Result<Self> {
        let headers = Self::read_headers(&mut reader)?;

        Ok(Self {
            reader,
            headers,
        })
    }

    pub fn read_headers(reader: &mut R) -> anyhow::Result<TIFFHeaders> {
        let mut buffer = [0; 12];

        // TIFF Header
        reader.seek(SeekFrom::Start(0))?;
        reader.read_exact(&mut buffer[..8])?;

        let endianness = Endianness::from_bytes(&buffer[..2])?;
        ensure!(endianness == Endianness::Small, "Only small endian supported so far.");
        ensure!(buffer[2] == 42, "Magic number is not a TIFF");

        // Offset to first IDF directory
        let mut next_idf_dir_offset = read_le_u32(&buffer[4..8]);

        let mut idf_dirs = vec![];
        while next_idf_dir_offset != 0 {
            // IDF directory
            reader.seek(SeekFrom::Start(next_idf_dir_offset as u64))?;
            reader.read_exact(&mut buffer[..2])?;
            let num_entries = read_le_u16(&buffer[..2]);

            // IDF entries
            let entries = (0..num_entries).map(|_| {
                reader.read_exact(&mut buffer[..12])?;
                IDFEntry::from_buf(&buffer[..12])
            }).collect::<anyhow::Result<Vec<IDFEntry>>>()?;

            // Offset to next IDF directory
            reader.read_exact(&mut buffer[..4])?;

            let idf_directory = IDFDirectory {
                offset: next_idf_dir_offset,
                entries,
            };
            idf_dirs.push(idf_directory);

            next_idf_dir_offset = read_le_u32(&buffer[..4]);
        }


        Ok(TIFFHeaders {
            endianness,
            directories: idf_dirs,
        })
    }

    pub fn read_entry(&mut self, entry: &IDFEntry) -> Result<Vec<u8>> {
        if entry.field_type.num_bytes() as u32 * entry.count <= 4 {
            // Value is stored in the offset
            return Ok(entry.offset.to_le_bytes().to_vec());
        }

        let mut buf = vec![0; entry.field_type.num_bytes() as usize * entry.count as usize];
        self.reader.seek(SeekFrom::Start(entry.offset as u64))?;
        self.reader.read_exact(&mut buf)?;

        Ok(buf)
    }

    pub fn get_tag_value(&mut self, layer: usize, tag_id: u16) -> Result<Vec<u8>> {
        let entry = self.headers.directories[layer].get_entry_by_tag_id(tag_id)?.clone();
        self.read_entry(&entry)
    }
}


#[cfg(test)]
mod tests {
    use std::fs::File;

    use crate::tiff::TIFFReader;

    use super::*;

    #[test]
    fn it_works() {
        let source_file = r"E:\datasets\tcga\images\hcmi_cmdc\0a5410d7-0f5c-4dda-986e-d857d176498f\HCM-CSHL-0366-C50-01A-01-S1-HE.30E0E448-BC32-4FCA-99F8-CF5E8C283352.svs";
        let file = File::open(source_file).expect("Failed to open file");
        let mut svs = TIFFReader::open(file).expect("Failed to read svs.");
        println!("{:?}", svs);
    }
}
