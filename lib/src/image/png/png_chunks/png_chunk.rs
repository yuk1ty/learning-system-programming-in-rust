mod png_chunk_type;

use std::{
    fmt::Display,
    io::{self, Read},
};

pub use png_chunk_type::PngChunkType;

/// Represents a chunk of PNG format.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct PngChunk {
    len: u32,
    typ: PngChunkType,
    crc: [u8; 4],
    data: Vec<u8>,
}

impl PngChunk {
    /// Creates "tEXt" chunk
    pub fn new_text_chunk(text: String) -> Self {
        let len = text.len() as u32;
        let typ = PngChunkType::new([b't', b'E', b'X', b't']);
        let data = text.into_bytes();
        let crc = {
            let mut hasher = crc32fast::Hasher::new();
            hasher.update(&data);
            let crc32 = hasher.finalize();
            crc32.to_be_bytes()
        };
        Self {
            len,
            typ,
            crc,
            data,
        }
    }

    /// # Returns
    ///
    /// None when `r` points to EOF
    pub(super) fn from_reader<R>(r: &mut R) -> io::Result<Option<Self>>
    where
        R: Read,
    {
        match Self::read_len(r) {
            Err(e) => match e.kind() {
                io::ErrorKind::UnexpectedEof => Ok(None),
                _ => Err(e),
            },
            Ok(len) => {
                let typ = Self::read_type(r)?;
                let data = Self::read_data(r, len)?;
                let crc = Self::read_crc(r)?;
                Ok(Some(Self {
                    len,
                    typ,
                    crc,
                    data,
                }))
            }
        }
    }

    pub(in super::super) fn into_vec(mut self) -> Vec<u8> {
        let mut bin = self.len.to_be_bytes().to_vec();
        bin.append(&mut self.typ.as_slice().to_vec());
        bin.append(&mut self.data);
        bin.append(&mut self.crc.to_vec());
        bin
    }

    fn read_len<R>(r: &mut R) -> io::Result<u32>
    where
        R: Read,
    {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_type<R>(r: &mut R) -> io::Result<PngChunkType>
    where
        R: Read,
    {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(PngChunkType::new(buf))
    }

    fn read_data<R>(r: &mut R, len: u32) -> io::Result<Vec<u8>>
    where
        R: Read,
    {
        let mut buf = Vec::<u8>::new();
        buf.resize(len as usize, 0);
        r.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn read_crc<R>(r: &mut R) -> io::Result<[u8; 4]>
    where
        R: Read,
    {
        let mut buf = [0u8; 4];
        r.read_exact(&mut buf)?;
        Ok(buf)
    }
}

impl Display for PngChunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = format!(
            "Chunk type: {}, Data len: {}, CRC: {:#X}",
            self.typ,
            self.len,
            u32::from_be_bytes(self.crc)
        );
        if self.typ.is_text() {
            s = format!(r#"{} - "{}""#, s, String::from_utf8(self.data.clone()).expect(
                "tEXt chunk can have printable Latin-1 text so conversion into UTF-8 may fail here",
            ));
        }
        write!(f, "{}", s)
    }
}
