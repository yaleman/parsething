use std::io::{BufRead, Read};

use crate::prelude::*;

#[derive(PackedStruct, Debug)]
#[packed_struct(bit_numbering = "msb0", size_bytes = "20", endian = "lsb")]
pub struct GgufHeader {
    #[packed_field(bytes = "0..=3")]
    version: u32,
    #[packed_field(bytes = "4..=11")]
    tensor_count: u64,
    #[packed_field(bytes = "12..=19")]
    metadata_kv_count: u64,
}

pub enum GgufParseError {
    InvalidHeader,
    UnsupportedVersion,
    CorruptedData,
}

#[repr(u32)]
pub enum GgmlType {
    F32 = 0,
    F16 = 1,
    Q40 = 2,
    Q41 = 3,
    //Q4_2 = 4, support has been removed
    //Q4_3 = 5, support has been removed
    Q50 = 6,
    Q51 = 7,
    Q80 = 8,
    Q81 = 9,
    Q2K = 10,
    Q3K = 11,
    Q4K = 12,
    Q5K = 13,
    Q6K = 14,
    Q8K = 15,
    IQ2XXS = 16,
    IQ2XS = 17,
    IQ3XXS = 18,
    IQ1S = 19,
    IQ4NL = 20,
    IQ3S = 21,
    IQ2S = 22,
    IQ4XS = 23,
    I8 = 24,
    I16 = 25,
    I32 = 26,
    I64 = 27,
    F64 = 28,
    IQ1M = 29,
    BF16 = 30,
    // Q4_0_4_4 = 31, support has been removed from gguf files
    // Q4_0_4_8 = 32,
    // Q4_0_8_8 = 33,
    TQ1_0 = 34,
    TQ2_0 = 35,
    // IQ4_NL_4_4 = 36,
    // IQ4_NL_4_8 = 37,
    // IQ4_NL_8_8 = 38,
    MXFP4 = 39, // MXFP4 (1 block)
    COUNT = 40,
}

impl TryFrom<u32> for GgmlType {
    type Error = ParseError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GgmlType::F32),
            1 => Ok(GgmlType::F16),
            2 => Ok(GgmlType::Q40),
            3 => Ok(GgmlType::Q41),
            6 => Ok(GgmlType::Q50),
            7 => Ok(GgmlType::Q51),
            8 => Ok(GgmlType::Q80),
            9 => Ok(GgmlType::Q81),
            10 => Ok(GgmlType::Q2K),
            11 => Ok(GgmlType::Q3K),
            12 => Ok(GgmlType::Q4K),
            13 => Ok(GgmlType::Q5K),
            14 => Ok(GgmlType::Q6K),
            15 => Ok(GgmlType::Q8K),
            16 => Ok(GgmlType::IQ2XXS),
            17 => Ok(GgmlType::IQ2XS),
            18 => Ok(GgmlType::IQ3XXS),
            19 => Ok(GgmlType::IQ1S),
            20 => Ok(GgmlType::IQ4NL),
            21 => Ok(GgmlType::IQ3S),
            22 => Ok(GgmlType::IQ2S),
            23 => Ok(GgmlType::IQ4XS),
            24 => Ok(GgmlType::I8),
            25 => Ok(GgmlType::I16),
            26 => Ok(GgmlType::I32),
            27 => Ok(GgmlType::I64),
            28 => Ok(GgmlType::F64),
            29 => Ok(GgmlType::IQ1M),
            30 => Ok(GgmlType::BF16),
            34 => Ok(GgmlType::TQ1_0),
            35 => Ok(GgmlType::TQ2_0),
            39 => Ok(GgmlType::MXFP4),
            _ => Err(ParseError::InvalidData),
        }
    }
}

#[repr(u32)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GgufMetadataValueType {
    // The value is a 8-bit unsigned integer.
    Uint8 = 0,
    // The value is a 8-bit signed integer.
    Int8 = 1,
    // The value is a 16-bit unsigned little-endian integer.
    Uint16 = 2,
    // The value is a 16-bit signed little-endian integer.
    Int16 = 3,
    // The value is a 32-bit unsigned little-endian integer.
    Uint32 = 4,
    // The value is a 32-bit signed little-endian integer.
    Int32 = 5,
    // The value is a 32-bit IEEE754 floating point number.
    Float32 = 6,
    // The value is a boolean.
    // 1-byte value where 0 is false and 1 is true.
    // Anything else is invalid, and should be treated as either the model being invalid or the reader being buggy.
    Bool = 7,
    // The value is a UTF-8 non-null-terminated string, with length prepended.
    String = 8,
    // The value is an array of other values, with the length and type prepended.
    // Arrays can be nested, and the length of the array is the number of elements in the array, not the number of bytes.
    Array = 9,
    // The value is a 64-bit unsigned little-endian integer.
    Uint64 = 10,
    // The value is a 64-bit signed little-endian integer.
    Int64 = 11,
    // The value is a 64-bit IEEE754 floating point number.
    Float64 = 12,
}

impl GgufMetadataValueType {
    pub fn size_in_bytes(&self) -> usize {
        match self {
            Self::Uint8 | Self::Int8 | Self::Bool => 1,
            Self::Uint16 | Self::Int16 => 2,
            Self::Uint32 | Self::Int32 | Self::Float32 => 4,
            Self::Uint64 | Self::Int64 | Self::Float64 => 8,
            Self::String | Self::Array => 0, // Variable size
        }
    }
}

impl TryFrom<u32> for GgufMetadataValueType {
    type Error = ParseError;
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(GgufMetadataValueType::Uint8),
            1 => Ok(GgufMetadataValueType::Int8),
            2 => Ok(GgufMetadataValueType::Uint16),
            3 => Ok(GgufMetadataValueType::Int16),
            4 => Ok(GgufMetadataValueType::Uint32),
            5 => Ok(GgufMetadataValueType::Int32),
            6 => Ok(GgufMetadataValueType::Float32),
            7 => Ok(GgufMetadataValueType::Bool),
            8 => Ok(GgufMetadataValueType::String),
            9 => Ok(GgufMetadataValueType::Array),
            10 => Ok(GgufMetadataValueType::Uint64),
            11 => Ok(GgufMetadataValueType::Int64),
            12 => Ok(GgufMetadataValueType::Float64),
            _ => Err(ParseError::InvalidData),
        }
    }
}

#[test]
fn test_gguf_metadatavalue() {
    assert_eq!(GgufMetadataValueType::Float64 as u32, 12);

    // let testval: u32 = 12;
    let testparsed = GgufMetadataValueType::try_from(12).expect("Failed to parse");
    assert_eq!(testparsed, GgufMetadataValueType::Float64);

    assert!(GgufMetadataValueType::try_from(99).is_err());
}

pub struct GgufTensor {
    pub name: String,
    pub n_dimensions: u32,
    pub dimensions: Vec<u64>,
    pub ggml_type: GgmlType,
    pub offset: u64,
}

pub enum GgufMetaValue {
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Binary(Vec<u8>),
    Array(Vec<GgufMetaValue>),
}

impl From<(GgufMetadataValueType, &Vec<u8>)> for GgufMetaValue {
    fn from((value_type, data): (GgufMetadataValueType, &Vec<u8>)) -> Self {
        match value_type {
            GgufMetadataValueType::Bool => GgufMetaValue::Bool(data[0] != 0),
            GgufMetadataValueType::Int8 => GgufMetaValue::Int(data[0] as i8 as i64),
            GgufMetadataValueType::Int16 => {
                let val = i16::from_le_bytes([data[0], data[1]]);
                GgufMetaValue::Int(val as i64)
            }
            GgufMetadataValueType::Int32 => {
                let val = i32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                GgufMetaValue::Int(val as i64)
            }
            GgufMetadataValueType::Int64 => {
                let val = i64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                GgufMetaValue::Int(val)
            }
            GgufMetadataValueType::Float32 => {
                let val = f32::from_le_bytes([data[0], data[1], data[2], data[3]]);
                GgufMetaValue::Float(val as f64)
            }
            GgufMetadataValueType::Float64 => {
                let val = f64::from_le_bytes([
                    data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
                ]);
                GgufMetaValue::Float(val)
            }
            _ => GgufMetaValue::Binary(data.to_vec()),
        }
    }
}

pub struct Gguf {
    pub header: GgufHeader,
    pub metadata: HashMap<String, GgufMetaValue>,
    pub tensors: Vec<GgufTensor>,
}

// gets a gguf string from the reader, which is a length-prefixed UTF-8 string
pub fn length_prefixed_string(reader: &mut impl BufRead) -> Result<String, ParseError> {
    let mut len_data = [0; 8];
    reader.read_exact(&mut len_data).map_err(|_err| {
        eprintln!("Failed to read 8 bytes for string length");
        ParseError::NeedMoreBytes
    })?;
    let str_len = u64::from_le_bytes(len_data) as usize;
    let mut str_data = vec![0; str_len];
    reader.read_exact(&mut str_data).map_err(|_err| {
        eprintln!("Failed to read {} bytes for string", str_len);
        ParseError::NeedMoreBytes
    })?;
    let s = String::from_utf8(str_data)?;
    Ok(s)
}

/// Reads a scalar value from the GGUF file.
fn read_scalar(
    reader: &mut impl BufRead,
    value_type: GgufMetadataValueType,
) -> Result<GgufMetaValue, ParseError> {
    if value_type == GgufMetadataValueType::String {
        return Ok(GgufMetaValue::String(length_prefixed_string(reader)?));
    }

    let mut data = vec![0; value_type.size_in_bytes()];
    reader
        .read_exact(&mut data)
        .map_err(|_err| ParseError::NeedMoreBytes)?;
    Ok(GgufMetaValue::from((value_type, &data)))
}

fn read_value(
    reader: &mut impl BufRead,
    value_type: GgufMetadataValueType,
) -> Result<GgufMetaValue, ParseError> {
    if value_type != GgufMetadataValueType::Array {
        return read_scalar(reader, value_type);
    }

    // we're handling an array!
    let mut element_type = [0; 4];
    reader.read_exact(&mut element_type)?;
    let element_type = GgufMetadataValueType::try_from(u32::from_le_bytes(element_type))?;

    let mut num_elements = [0; 8];
    reader.read_exact(&mut num_elements)?;
    let num_elements = u64::from_le_bytes(num_elements) as usize;

    let mut res = Vec::new();
    for _ in 0..num_elements {
        let element_value = read_scalar(reader, element_type)?;
        res.push(element_value);
    }
    Ok(GgufMetaValue::Array(res))
}

fn parse_tensor(reader: &mut impl BufRead) -> Result<GgufTensor, ParseError> {
    let name = length_prefixed_string(reader)?;

    let mut n_dims_data = [0; 4];
    reader.read_exact(&mut n_dims_data)?;
    let n_dimensions = u32::from_le_bytes(n_dims_data);

    let mut dimensions = Vec::new();
    for _ in 0..n_dimensions {
        let mut dim_data = [0; 8];
        reader.read_exact(&mut dim_data)?;
        let dim = u64::from_le_bytes(dim_data);
        dimensions.push(dim);
    }

    let mut ggml_type_data = [0; 4];
    reader.read_exact(&mut ggml_type_data)?;
    let ggml_type = GgmlType::try_from(u32::from_le_bytes(ggml_type_data))
        .map_err(|_| ParseError::InvalidData)?;

    let mut offset_data = [0; 8];
    reader.read_exact(&mut offset_data)?;
    let offset = u64::from_le_bytes(offset_data);

    Ok(GgufTensor {
        name,
        n_dimensions,
        dimensions,
        ggml_type,
        offset,
    })
}

impl ParseThing for Gguf {
    fn parse(data: &mut impl BufRead) -> Result<Box<Self>, ParseError> {
        let mut firstfour_data = [0; 4];
        data.read(&mut firstfour_data).map_err(|_| {
            ParseError::InvalidHeader("Failed to read first 4 bytes from input".to_string())
        })?;
        if &firstfour_data != b"GGUF" {
            return Err(ParseError::InvalidHeader(
                "Invalid magic bytes, should start with 'GGUF'".to_string(),
            ));
        }

        let mut headerbytes = [0; 20];
        data.take(20).read_exact(&mut headerbytes).map_err(|_| {
            ParseError::InvalidHeader("Failed to read 20 bytes from input".to_string())
        })?;
        let header = GgufHeader::unpack_from_slice(&headerbytes).map_err(|_| {
            ParseError::InvalidHeader("Failed to unpack header from bytes".to_string())
        })?;
        #[cfg(any(test, debug_assertions))]
        eprintln!("Header: {:?}", header);

        let mut metadata: HashMap<String, GgufMetaValue> = HashMap::new();

        for _mdnum in 0..header.metadata_kv_count {
            let key = length_prefixed_string(data)?;
            #[cfg(any(test, debug_assertions))]
            eprintln!("Metadata key: {}", key);

            // get the next four bytes which is the value type
            let mut valtype_data = [0; 4];
            data.read_exact(&mut valtype_data).map_err(|_| {
                ParseError::InvalidHeader(
                    "Failed to read 4 bytes for metadata value type".to_string(),
                )
            })?;
            let value_type: GgufMetadataValueType =
                GgufMetadataValueType::try_from(u32::from_le_bytes(valtype_data))?;
            #[cfg(any(test, debug_assertions))]
            eprintln!("Metadata value type: {:?}", value_type);
            let value = read_value(data, value_type)?;
            metadata.insert(key, value);
        }
        let mut tensors: Vec<GgufTensor> = Vec::new();
        for _tensor_num in 0..header.tensor_count {
            let tensor = parse_tensor(data)?;
            #[cfg(any(test, debug_assertions))]
            eprintln!("Parsed tensor: {:?}", tensor.name);
            tensors.push(tensor);
        }

        Ok(Box::new(Gguf {
            header,
            metadata,
            tensors,
        }))
    }
    fn verify(&self) -> Result<bool, ParseError> {
        // until someone else uses another version!
        if self.header.version != 3 {
            return Err(ParseError::InvalidHeader(format!(
                "Unsupported GGUF version, should be 3, got {}",
                self.header.version
            )));
        }
        Ok(true)
    }
}

#[test]
fn test_gguf_header_parsing() {
    // let (filename, expected_metadata_count, expected_tensor_count ) = ("Baguettotron-Q8_0.gguf", 31, 722);
    let (filename, expected_metadata_count, expected_tensor_count) =
        ("ggml-vocab-llama.gguf", 17, 0);
    let testdata =
        std::fs::read(format!("test_data/gguf/{filename}")).expect("Failed to read test GGUF file");
    eprintln!("bytes: {:?}", &testdata[0..24]);
    let mut cursor = std::io::Cursor::new(testdata);

    let gguf = Gguf::parse(&mut cursor).expect("Failed to parse GGUF header");
    assert_eq!(gguf.header.version, 3, "should be version 3");
    assert_eq!(gguf.header.tensor_count, expected_tensor_count);
    assert_eq!(gguf.tensors.len(), expected_tensor_count as usize);
    assert_eq!(gguf.header.metadata_kv_count, expected_metadata_count);
    assert_eq!(gguf.metadata.len(), expected_metadata_count as usize);
    assert!(gguf.verify().expect("Failed to verify GGUF"));
}
