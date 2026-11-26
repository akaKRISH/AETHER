use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

use serde::{Deserialize, Serialize, de};

use crate::Result;

const MAX_HEADER: u64 = 16 * 1024 * 1024;
const MAX_TENSORS: usize = 32_768;
const MAX_METADATA: u64 = 4096;
const MAX_ARRAY: u64 = 524_288;
const MAX_TEXT: u64 = 256 * 1024;
const SAMPLE: usize = 32;

#[derive(Debug, Serialize)]
pub(super) struct Inspection {
    format: &'static str,
    file_bytes: u64,
    payload_bytes: u64,
    stored_elements: u64,
    tensor_count: usize,
    dtype_tensor_counts: BTreeMap<String, usize>,
    tensors: Vec<Tensor>,
    omitted_tensors: usize,
    unsupported_dtypes: usize,
    metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
struct Tensor {
    name: String,
    dtype: String,
    shape: Vec<u64>,
    offset: u64,
    bytes: Option<u64>,
}

pub(super) fn inspect(file: &mut File, bytes: u64) -> Result<Inspection> {
    let mut prefix = [0; 4];
    file.read_exact(&mut prefix)?;
    file.seek(SeekFrom::Start(0))?;
    let mut reader = BufReader::new(file).take(MAX_HEADER);
    if &prefix == b"GGUF" {
        gguf(&mut reader, bytes)
    } else {
        safetensors(&mut reader, bytes)
    }
}

fn number<const N: usize>(r: &mut impl Read) -> std::io::Result<[u8; N]> {
    let mut bytes = [0; N];
    r.read_exact(&mut bytes)?;
    Ok(bytes)
}

fn u32le(r: &mut impl Read) -> std::io::Result<u32> {
    Ok(u32::from_le_bytes(number(r)?))
}

fn u64le(r: &mut impl Read) -> std::io::Result<u64> {
    Ok(u64::from_le_bytes(number(r)?))
}

fn text(r: &mut impl Read, limit: u64) -> Result<String> {
    let len = u64le(r)?;
    if len > limit {
        return Err("GGUF string exceeds the inspection limit".into());
    }
    let mut bytes = vec![0; len as usize];
    r.read_exact(&mut bytes)?;
    Ok(String::from_utf8(bytes)?)
}

fn skip(r: &mut impl Read, bytes: u64) -> Result<()> {
    if std::io::copy(&mut r.take(bytes), &mut std::io::sink())? != bytes {
        return Err("truncated GGUF metadata".into());
    }
    Ok(())
}

fn scalar_size(kind: u32) -> Option<u64> {
    match kind {
        0 | 1 | 7 => Some(1),
        2 | 3 => Some(2),
        4..=6 => Some(4),
        10..=12 => Some(8),
        _ => None,
    }
}

fn gguf_metadata(r: &mut impl Read, count: u64) -> Result<BTreeMap<String, serde_json::Value>> {
    let mut result = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for _ in 0..count {
        let key = text(r, 256)?;
        if !seen.insert(key.clone()) {
            return Err("duplicate GGUF metadata key".into());
        }
        let kind = u32le(r)?;
        let value = match (key.as_str(), kind) {
            (
                "general.alignment"
                | "general.file_type"
                | "general.quantization_version"
                | "tokenizer.ggml.bos_token_id"
                | "tokenizer.ggml.eos_token_id",
                4,
            ) => Some(serde_json::json!(u32le(r)?)),
            ("general.alignment", _) => return Err("invalid GGUF alignment type".into()),
            (
                "general.architecture"
                | "general.name"
                | "general.license"
                | "general.source.url"
                | "general.source.repo_url"
                | "tokenizer.ggml.model"
                | "tokenizer.ggml.pre",
                8,
            ) => Some(serde_json::json!(text(r, 4096)?)),
            (_, 8) => {
                let len = u64le(r)?;
                if len > MAX_TEXT {
                    return Err("GGUF string exceeds the inspection limit".into());
                }
                skip(r, len)?;
                None
            }
            (_, 9) => {
                let element = u32le(r)?;
                let count = u64le(r)?;
                if count > MAX_ARRAY {
                    return Err("GGUF array exceeds the inspection limit".into());
                }
                if element == 8 {
                    for _ in 0..count {
                        let size = u64le(r)?;
                        if size > MAX_TEXT {
                            return Err("GGUF array string exceeds the inspection limit".into());
                        }
                        skip(r, size)?;
                    }
                } else {
                    skip(
                        r,
                        count * scalar_size(element).ok_or("unsupported GGUF array element")?,
                    )?;
                }
                matches!(
                    key.as_str(),
                    "tokenizer.ggml.tokens" | "tokenizer.ggml.merges"
                )
                .then(|| serde_json::json!({"count": count, "value_type": element}))
            }
            (_, kind) => {
                skip(r, scalar_size(kind).ok_or("unknown GGUF metadata type")?)?;
                None
            }
        };
        if let Some(value) = value {
            result.insert(key, value);
        }
    }
    Ok(result)
}

// GGML type codes and block sizes for the supported layouts.
fn ggml(code: u32) -> Option<(&'static str, u64, u64)> {
    Some(match code {
        0 => ("F32", 1, 4),
        1 => ("F16", 1, 2),
        2 => ("Q4_0", 32, 18),
        3 => ("Q4_1", 32, 20),
        6 => ("Q5_0", 32, 22),
        7 => ("Q5_1", 32, 24),
        8 => ("Q8_0", 32, 34),
        10 => ("Q2_K", 256, 84),
        11 => ("Q3_K", 256, 110),
        12 => ("Q4_K", 256, 144),
        13 => ("Q5_K", 256, 176),
        14 => ("Q6_K", 256, 210),
        15 => ("Q8_K", 256, 292),
        24 => ("I8", 1, 1),
        25 => ("I16", 1, 2),
        26 => ("I32", 1, 4),
        27 => ("I64", 1, 8),
        28 => ("F64", 1, 8),
        30 => ("BF16", 1, 2),
        _ => return None,
    })
}

fn elements(shape: &[u64]) -> Result<u64> {
    if shape.contains(&0) {
        return Ok(0);
    }
    shape.iter().try_fold(1u64, |product, n| {
        product
            .checked_mul(*n)
            .ok_or_else(|| "tensor dimensions overflow".into())
    })
}

fn gguf(r: &mut std::io::Take<impl Read>, file_bytes: u64) -> Result<Inspection> {
    if number::<4>(r)? != *b"GGUF" || !matches!(u32le(r)?, 2 | 3) {
        return Err("only little-endian GGUF v2 and v3 are supported".into());
    }
    let count = u64le(r)?;
    let metadata_count = u64le(r)?;
    if count == 0 || count > MAX_TENSORS as u64 || metadata_count > MAX_METADATA {
        return Err("GGUF descriptor limit exceeded".into());
    }
    let metadata = gguf_metadata(r, metadata_count)?;
    let alignment = metadata
        .get("general.alignment")
        .and_then(serde_json::Value::as_u64)
        .unwrap_or(32);
    if alignment == 0 || alignment > 4096 || !alignment.is_multiple_of(8) {
        return Err("invalid GGUF alignment".into());
    }
    let mut tensors = Vec::new();
    let mut names = BTreeSet::new();
    for _ in 0..count {
        let name = text(r, 1024)?;
        let rank = u32le(r)?;
        if !names.insert(name.clone()) || !(1..=4).contains(&rank) {
            return Err("duplicate GGUF tensor or invalid rank".into());
        }
        let shape = (0..rank)
            .map(|_| u64le(r))
            .collect::<std::io::Result<Vec<_>>>()?;
        let n = elements(&shape)?;
        let code = u32le(r)?;
        let offset = u64le(r)?;
        if n == 0 || !offset.is_multiple_of(alignment) {
            return Err("invalid GGUF tensor extent".into());
        }
        let (dtype, bytes) = match ggml(code) {
            Some((name, block, size)) => {
                if !shape[0].is_multiple_of(block) {
                    return Err("GGUF quantized row is not block aligned".into());
                }
                (
                    name.into(),
                    Some(
                        (n / block)
                            .checked_mul(size)
                            .ok_or("tensor byte size overflow")?,
                    ),
                )
            }
            None => (format!("ggml:{code}"), None),
        };
        tensors.push(Tensor {
            name,
            dtype,
            shape,
            offset,
            bytes,
        });
    }
    let data_start = (MAX_HEADER - r.limit()).div_ceil(alignment) * alignment;
    let payload = file_bytes
        .checked_sub(data_start)
        .ok_or("truncated GGUF alignment padding")?;
    summarize("gguf", file_bytes, payload, tensors, metadata)
}

// Serde's ordinary maps overwrite duplicate keys, which the format forbids.
struct UniqueMap<V>(BTreeMap<String, V>);

impl<'de, V: Deserialize<'de>> Deserialize<'de> for UniqueMap<V> {
    fn deserialize<D: de::Deserializer<'de>>(
        deserializer: D,
    ) -> std::result::Result<Self, D::Error> {
        struct Visitor<V>(std::marker::PhantomData<V>);
        impl<'de, V: Deserialize<'de>> de::Visitor<'de> for Visitor<V> {
            type Value = UniqueMap<V>;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a bounded map with unique keys")
            }
            fn visit_map<A: de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut values = BTreeMap::new();
                while let Some((key, value)) = map.next_entry::<String, V>()? {
                    if key.len() > 1024
                        || values.len() > MAX_TENSORS
                        || values.insert(key, value).is_some()
                    {
                        return Err(de::Error::custom(
                            "duplicate or oversized SafeTensors descriptor",
                        ));
                    }
                }
                Ok(UniqueMap(values))
            }
        }
        deserializer.deserialize_map(Visitor(std::marker::PhantomData))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SafeTensor {
    dtype: String,
    #[serde(deserialize_with = "bounded_shape")]
    shape: Vec<u64>,
    data_offsets: [u64; 2],
}

fn bounded_shape<'de, D: de::Deserializer<'de>>(d: D) -> std::result::Result<Vec<u64>, D::Error> {
    struct Shape;
    impl<'de> de::Visitor<'de> for Shape {
        type Value = Vec<u64>;
        fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.write_str("at most eight dimensions")
        }
        fn visit_seq<A: de::SeqAccess<'de>>(
            self,
            mut seq: A,
        ) -> std::result::Result<Self::Value, A::Error> {
            let mut shape = Vec::new();
            while let Some(n) = seq.next_element()? {
                if shape.len() == 8 {
                    return Err(de::Error::custom(
                        "SafeTensors rank exceeds the inspection limit",
                    ));
                }
                shape.push(n);
            }
            Ok(shape)
        }
    }
    d.deserialize_seq(Shape)
}

#[derive(Default)]
struct SafeHeader {
    fields: Option<UniqueMap<String>>,
    tensors: BTreeMap<String, SafeTensor>,
}

impl<'de> Deserialize<'de> for SafeHeader {
    fn deserialize<D: de::Deserializer<'de>>(d: D) -> std::result::Result<Self, D::Error> {
        struct Header;
        impl<'de> de::Visitor<'de> for Header {
            type Value = SafeHeader;
            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a bounded SafeTensors header")
            }
            fn visit_map<A: de::MapAccess<'de>>(
                self,
                mut map: A,
            ) -> std::result::Result<Self::Value, A::Error> {
                let mut header = SafeHeader::default();
                while let Some(key) = map.next_key::<String>()? {
                    if key == "__metadata__" && header.fields.is_none() {
                        header.fields = Some(map.next_value()?);
                    } else if key == "__metadata__"
                        || key.len() > 1024
                        || header.tensors.len() >= MAX_TENSORS
                        || header.tensors.contains_key(&key)
                    {
                        return Err(de::Error::custom(
                            "duplicate or oversized SafeTensors descriptor",
                        ));
                    } else {
                        header.tensors.insert(key, map.next_value()?);
                    }
                }
                Ok(header)
            }
        }
        d.deserialize_map(Header)
    }
}

fn safetensors(r: &mut impl Read, file_bytes: u64) -> Result<Inspection> {
    let header = u64le(r)?;
    if !(2..=MAX_HEADER - 8).contains(&header) || header + 8 > file_bytes {
        return Err("invalid SafeTensors header length".into());
    }
    let mut raw = vec![0; header as usize];
    r.read_exact(&mut raw)?;
    if raw[0] != b'{' {
        return Err("invalid SafeTensors header".into());
    }
    let parsed: SafeHeader = serde_json::from_slice(&raw)?;
    let mut metadata = BTreeMap::new();
    let mut tensors = Vec::new();
    if let Some(UniqueMap(fields)) = parsed.fields {
        for key in [
            "architecture",
            "format",
            "license",
            "source",
            "tokenizer",
            "quantization",
        ] {
            if let Some(value) = fields.get(key) {
                if value.len() > 4096 {
                    return Err("SafeTensors metadata exceeds the inspection limit".into());
                }
                metadata.insert(key.into(), serde_json::json!(value));
            }
        }
    }
    for (
        name,
        SafeTensor {
            dtype,
            shape,
            data_offsets: [offset, end],
        },
    ) in parsed.tensors
    {
        let bytes = end
            .checked_sub(offset)
            .ok_or("invalid SafeTensors offsets")?;
        let width = match dtype.as_str() {
            "BOOL" | "I8" | "U8" | "F8_E4M3" | "F8_E5M2" | "F8_E8M0" => 1,
            "I16" | "U16" | "F16" | "BF16" => 2,
            "I32" | "U32" | "F32" => 4,
            "I64" | "U64" | "F64" => 8,
            _ => return Err("unsupported SafeTensors dtype".into()),
        };
        if elements(&shape)?.checked_mul(width) != Some(bytes) {
            return Err("SafeTensors shape and byte range disagree".into());
        }
        tensors.push(Tensor {
            name,
            dtype,
            shape,
            offset,
            bytes: Some(bytes),
        });
    }
    summarize(
        "safetensors",
        file_bytes,
        file_bytes - header - 8,
        tensors,
        metadata,
    )
}

fn summarize(
    format: &'static str,
    file_bytes: u64,
    payload_bytes: u64,
    mut tensors: Vec<Tensor>,
    metadata: BTreeMap<String, serde_json::Value>,
) -> Result<Inspection> {
    tensors.sort_by_key(|t| (t.offset, t.bytes.unwrap_or(1)));
    let mut end = 0;
    let mut stored_elements = 0u64;
    let mut unsupported_dtypes = 0;
    let mut dtype_tensor_counts = BTreeMap::new();
    for tensor in &tensors {
        if tensor.offset < end || (format == "safetensors" && tensor.offset != end) {
            return Err("overlapping or unindexed tensor regions".into());
        }
        unsupported_dtypes += usize::from(tensor.bytes.is_none());
        end = tensor
            .offset
            .checked_add(tensor.bytes.unwrap_or(1))
            .filter(|end| *end <= payload_bytes)
            .ok_or("tensor extent exceeds its payload")?;
        stored_elements = stored_elements
            .checked_add(elements(&tensor.shape)?)
            .ok_or("total tensor element count overflow")?;
        *dtype_tensor_counts.entry(tensor.dtype.clone()).or_insert(0) += 1;
    }
    if format == "safetensors" && end != payload_bytes {
        return Err("unindexed SafeTensors payload".into());
    }
    let tensor_count = tensors.len();
    if tensor_count == 0 || tensor_count > MAX_TENSORS {
        return Err("tensor descriptor count falls outside the inspection limit".into());
    }
    tensors.truncate(SAMPLE);
    Ok(Inspection {
        format,
        file_bytes,
        payload_bytes,
        stored_elements,
        tensor_count,
        dtype_tensor_counts,
        tensors,
        omitted_tensors: tensor_count.saturating_sub(SAMPLE),
        unsupported_dtypes,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn safe(header: &str, payload: u64) -> Result<Inspection> {
        let raw = (header.len() as u64)
            .to_le_bytes()
            .into_iter()
            .chain(header.bytes())
            .collect::<Vec<_>>();
        safetensors(&mut raw.as_slice(), raw.len() as u64 + payload)
    }

    #[test]
    fn safetensors_validates_scalars_empty_shapes_and_exact_coverage() {
        let result = safe(r#"{"__metadata__":{"license":"fixture"},"empty":{"dtype":"F16","shape":[0],"data_offsets":[0,0]},"scalar":{"dtype":"F32","shape":[],"data_offsets":[0,4]}}"#, 4).unwrap();
        assert_eq!(result.tensor_count, 2);
        assert_eq!(result.stored_elements, 1);
        for (header, payload) in [
            (
                r#"{"a":{"dtype":"I8","shape":[1],"data_offsets":[0,1]},"a":{"dtype":"I8","shape":[1],"data_offsets":[0,1]}}"#,
                1,
            ),
            (
                r#"{"a":{"dtype":"I8","shape":[1],"data_offsets":[1,2]}}"#,
                2,
            ),
            (
                r#"{"a":{"dtype":"I8","shape":[1],"data_offsets":[0,1]},"b":{"dtype":"I8","shape":[1],"data_offsets":[0,1]}}"#,
                1,
            ),
            (
                r#"{"a":{"dtype":"F64","shape":[18446744073709551615,2],"data_offsets":[0,8]}}"#,
                8,
            ),
            (
                r#"{"a":{"dtype":"F16","shape":[1],"data_offsets":[0,1]}}"#,
                1,
            ),
            (r#"{"__metadata__":{"x":[1]}}"#, 0),
            (
                r#"{"a":{"dtype":"I8","shape":[1,1,1,1,1,1,1,1,1],"data_offsets":[0,1]}}"#,
                1,
            ),
        ] {
            assert!(safe(header, payload).is_err());
        }
        assert!(safetensors(&mut (MAX_HEADER + 1).to_le_bytes().as_slice(), u64::MAX).is_err());
    }

    #[test]
    fn gguf_quantized_extent_has_exact_blocks_and_bounded_descriptors() {
        let mut bytes = b"GGUF".to_vec();
        bytes.extend(3u32.to_le_bytes());
        bytes.extend(1u64.to_le_bytes());
        bytes.extend(0u64.to_le_bytes());
        bytes.extend(1u64.to_le_bytes());
        bytes.push(b'w');
        bytes.extend(1u32.to_le_bytes());
        bytes.extend(256u64.to_le_bytes());
        bytes.extend(12u32.to_le_bytes());
        bytes.extend(0u64.to_le_bytes());
        let read = |input: &[u8], length| gguf(&mut input.take(MAX_HEADER), length);
        let parsed = read(&bytes, 64 + 144).unwrap();
        assert_eq!(parsed.tensors[0].bytes, Some(144));
        assert_eq!(parsed.tensors[0].dtype, "Q4_K");
        assert!(read(&bytes, 64 + 143).is_err());
        assert!(read(&bytes[..bytes.len() - 1], 64 + 144).is_err());
        bytes[37..45].copy_from_slice(&255u64.to_le_bytes());
        assert!(read(&bytes, 64 + 144).is_err());
        bytes[8..16].copy_from_slice(&u64::MAX.to_le_bytes());
        assert!(read(&bytes, u64::MAX).is_err());
    }
}
