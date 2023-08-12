use byteorder::{NativeEndian, ReadBytesExt};
use std::io::Cursor;

// UEFI Spec 2.8, 3.1.3.
// TODO: add file_path_list.
#[derive(Debug, PartialEq)]
pub struct EFILoadOpt {
    pub attributes: u32,
    pub description: String,
    pub optional_data: Vec<u8>,
}

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    #[error("attributes: {0}")]
    Attributes(std::io::Error),
    #[error("file path list length: {0}")]
    FilePathListLength(std::io::Error),
    #[error("description reading: {0}")]
    DescriptionReading(std::io::Error),
    #[error("description parsing: {0}")]
    DescriptionParsing(std::string::FromUtf16Error),
}

impl EFILoadOpt {
    pub fn decode(buf: &[u8]) -> Result<Self, DecodeError> {
        let mut cursor = Cursor::new(buf);
        let attributes = cursor
            .read_u32::<NativeEndian>()
            .map_err(DecodeError::Attributes)?;
        let file_path_list_length = cursor
            .read_u16::<NativeEndian>()
            .map_err(DecodeError::FilePathListLength)?;
        let description_start = cursor.position() as usize;

        let vec_capacity = (buf.len() - description_start + 1) >> 1;
        let mut description_buf: Vec<u16> = Vec::with_capacity(vec_capacity);
        loop {
            let c = cursor
                .read_u16::<NativeEndian>()
                .map_err(DecodeError::DescriptionReading)?;
            if c == 0 {
                break;
            }
            description_buf.push(c);
        }

        let description =
            String::from_utf16(&description_buf).map_err(DecodeError::DescriptionParsing)?;

        let optional_data_start = cursor.position() as usize + file_path_list_length as usize;
        let optional_data: Vec<u8> = Vec::from(&buf[optional_data_start..]);

        Ok(EFILoadOpt {
            attributes: attributes,
            description: description,
            optional_data: optional_data,
        })
    }
}

#[test]
fn decode_test() {
    let buf: Vec<u8> = vec![
        1, 0, 0, 0, 98, 0, 117, 0, 98, 0, 117, 0, 110, 0, 116, 0, 117, 0, 0, 0, 4, 1, 42, 0, 6, 0,
        0, 0, 0, 8, 132, 56, 1, 0, 0, 0, 0, 0, 32, 0, 0, 0, 0, 0, 17, 231, 186, 229, 231, 20, 31,
        76, 149, 26, 169, 112, 74, 106, 62, 66, 2, 2, 4, 4, 52, 0, 92, 0, 69, 0, 70, 0, 73, 0, 92,
        0, 85, 0, 66, 0, 85, 0, 78, 0, 84, 0, 85, 0, 92, 0, 83, 0, 72, 0, 73, 0, 77, 0, 88, 0, 54,
        0, 52, 0, 46, 0, 69, 0, 70, 0, 73, 0, 0, 0, 127, 255, 4, 0, 0, 0, 66, 79,
    ];
    assert_eq!(
        EFILoadOpt::decode(&buf).unwrap(),
        EFILoadOpt {
            attributes: 1,
            description: "ubuntu".to_owned(),
            optional_data: vec![0, 0, 66, 79],
        }
    );
}
