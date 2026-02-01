//! Binary format for GFA files using bincode.
//!
//! This module provides serialization and deserialization of GFA objects
//! to a compact binary format with the `.gfabin` extension.

use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::Path;

use bincode::{Decode, Encode};

use crate::cigar::{CIGARPair, CIGAR};
use crate::gfa::{
    Containment, Header, Jump, Link, Orientation, Path as GfaPath, Segment,
    Walk, GFA,
};
use crate::optfields::{OptField, OptFieldVal, OptFields};

/// Magic bytes for the binary format header
const MAGIC: &[u8; 6] = b"GFABIN";

/// Current format version
const VERSION: u8 = 1;

/// Error type for binary format operations
#[derive(Debug)]
pub enum BinaryFormatError {
    Io(std::io::Error),
    Encode(bincode::error::EncodeError),
    Decode(bincode::error::DecodeError),
    InvalidMagic,
    UnsupportedVersion(u8),
}

impl std::fmt::Display for BinaryFormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BinaryFormatError::Io(e) => write!(f, "IO error: {}", e),
            BinaryFormatError::Encode(e) => write!(f, "Encode error: {}", e),
            BinaryFormatError::Decode(e) => write!(f, "Decode error: {}", e),
            BinaryFormatError::InvalidMagic => write!(f, "Invalid magic bytes"),
            BinaryFormatError::UnsupportedVersion(v) => {
                write!(f, "Unsupported format version: {}", v)
            }
        }
    }
}

impl std::error::Error for BinaryFormatError {}

impl From<std::io::Error> for BinaryFormatError {
    fn from(e: std::io::Error) -> Self {
        BinaryFormatError::Io(e)
    }
}

impl From<bincode::error::EncodeError> for BinaryFormatError {
    fn from(e: bincode::error::EncodeError) -> Self {
        BinaryFormatError::Encode(e)
    }
}

impl From<bincode::error::DecodeError> for BinaryFormatError {
    fn from(e: bincode::error::DecodeError) -> Self {
        BinaryFormatError::Decode(e)
    }
}

// Binary-compatible representations of GFA types

#[derive(Encode, Decode)]
struct BinOrientation(u8);

impl From<Orientation> for BinOrientation {
    fn from(o: Orientation) -> Self {
        BinOrientation(match o {
            Orientation::Forward => 0,
            Orientation::Backward => 1,
        })
    }
}

impl From<BinOrientation> for Orientation {
    fn from(o: BinOrientation) -> Self {
        match o.0 {
            0 => Orientation::Forward,
            _ => Orientation::Backward,
        }
    }
}

#[derive(Encode, Decode)]
struct BinCIGARPair(u32);

impl From<CIGARPair> for BinCIGARPair {
    fn from(p: CIGARPair) -> Self {
        BinCIGARPair(p.into())
    }
}

impl From<BinCIGARPair> for CIGARPair {
    fn from(p: BinCIGARPair) -> Self {
        CIGARPair::from(p.0)
    }
}

#[derive(Encode, Decode)]
struct BinCIGAR(Vec<BinCIGARPair>);

impl From<&CIGAR> for BinCIGAR {
    fn from(c: &CIGAR) -> Self {
        BinCIGAR(c.0.iter().copied().map(BinCIGARPair::from).collect())
    }
}

impl From<BinCIGAR> for CIGAR {
    fn from(c: BinCIGAR) -> Self {
        CIGAR(c.0.into_iter().map(CIGARPair::from).collect())
    }
}

#[derive(Encode, Decode)]
enum BinOptFieldVal {
    A(u8),
    Int(i64),
    Float(f32),
    Z(Vec<u8>),
    J(Vec<u8>),
    H(Vec<u32>),
    BInt(Vec<i64>),
    BFloat(Vec<f32>),
}

impl From<&OptFieldVal> for BinOptFieldVal {
    fn from(v: &OptFieldVal) -> Self {
        match v {
            OptFieldVal::A(x) => BinOptFieldVal::A(*x),
            OptFieldVal::Int(x) => BinOptFieldVal::Int(*x),
            OptFieldVal::Float(x) => BinOptFieldVal::Float(*x),
            OptFieldVal::Z(x) => BinOptFieldVal::Z(x.clone()),
            OptFieldVal::J(x) => BinOptFieldVal::J(x.clone()),
            OptFieldVal::H(x) => BinOptFieldVal::H(x.clone()),
            OptFieldVal::BInt(x) => BinOptFieldVal::BInt(x.clone()),
            OptFieldVal::BFloat(x) => BinOptFieldVal::BFloat(x.clone()),
        }
    }
}

impl From<BinOptFieldVal> for OptFieldVal {
    fn from(v: BinOptFieldVal) -> Self {
        match v {
            BinOptFieldVal::A(x) => OptFieldVal::A(x),
            BinOptFieldVal::Int(x) => OptFieldVal::Int(x),
            BinOptFieldVal::Float(x) => OptFieldVal::Float(x),
            BinOptFieldVal::Z(x) => OptFieldVal::Z(x),
            BinOptFieldVal::J(x) => OptFieldVal::J(x),
            BinOptFieldVal::H(x) => OptFieldVal::H(x),
            BinOptFieldVal::BInt(x) => OptFieldVal::BInt(x),
            BinOptFieldVal::BFloat(x) => OptFieldVal::BFloat(x),
        }
    }
}

#[derive(Encode, Decode)]
struct BinOptField {
    tag: [u8; 2],
    value: BinOptFieldVal,
}

impl From<&OptField> for BinOptField {
    fn from(f: &OptField) -> Self {
        BinOptField {
            tag: f.tag,
            value: BinOptFieldVal::from(&f.value),
        }
    }
}

impl From<BinOptField> for OptField {
    fn from(f: BinOptField) -> Self {
        OptField {
            tag: f.tag,
            value: OptFieldVal::from(f.value),
        }
    }
}

#[derive(Encode, Decode)]
struct BinHeader {
    version: Option<Vec<u8>>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinSegment {
    name: Vec<u8>,
    sequence: Vec<u8>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinLink {
    from_segment: Vec<u8>,
    from_orient: BinOrientation,
    to_segment: Vec<u8>,
    to_orient: BinOrientation,
    overlap: Vec<u8>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinContainment {
    container_name: Vec<u8>,
    container_orient: BinOrientation,
    contained_name: Vec<u8>,
    contained_orient: BinOrientation,
    pos: u64,
    overlap: Vec<u8>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinPath {
    path_name: Vec<u8>,
    segment_names: Vec<u8>,
    overlaps: Vec<Option<BinCIGAR>>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinWalk {
    sample_id: Vec<u8>,
    hap_index: u64,
    seq_id: Vec<u8>,
    seq_start: Option<u64>,
    seq_end: Option<u64>,
    segment_path: Vec<(Vec<u8>, BinOrientation)>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinJump {
    from_segment: Vec<u8>,
    from_orient: BinOrientation,
    to_segment: Vec<u8>,
    to_orient: BinOrientation,
    distance: Option<i64>,
    optional: Vec<BinOptField>,
}

#[derive(Encode, Decode)]
struct BinGFA {
    header: BinHeader,
    segments: Vec<BinSegment>,
    links: Vec<BinLink>,
    containments: Vec<BinContainment>,
    paths: Vec<BinPath>,
    walks: Vec<BinWalk>,
    jumps: Vec<BinJump>,
}

// Helper functions to convert optional fields
fn opt_fields_to_bin<T: OptFields>(opt: &T) -> Vec<BinOptField> {
    opt.fields().iter().map(BinOptField::from).collect()
}

fn bin_to_opt_fields(bin: Vec<BinOptField>) -> Vec<OptField> {
    bin.into_iter().map(OptField::from).collect()
}

// Conversion implementations
impl<T: OptFields> From<&GFA<Vec<u8>, T>> for BinGFA {
    fn from(gfa: &GFA<Vec<u8>, T>) -> Self {
        BinGFA {
            header: BinHeader {
                version: gfa.header.version.clone(),
                optional: opt_fields_to_bin(&gfa.header.optional),
            },
            segments: gfa
                .segments
                .iter()
                .map(|s| BinSegment {
                    name: s.name.clone(),
                    sequence: s.sequence.clone(),
                    optional: opt_fields_to_bin(&s.optional),
                })
                .collect(),
            links: gfa
                .links
                .iter()
                .map(|l| BinLink {
                    from_segment: l.from_segment.clone(),
                    from_orient: l.from_orient.into(),
                    to_segment: l.to_segment.clone(),
                    to_orient: l.to_orient.into(),
                    overlap: l.overlap.clone(),
                    optional: opt_fields_to_bin(&l.optional),
                })
                .collect(),
            containments: gfa
                .containments
                .iter()
                .map(|c| BinContainment {
                    container_name: c.container_name.clone(),
                    container_orient: c.container_orient.into(),
                    contained_name: c.contained_name.clone(),
                    contained_orient: c.contained_orient.into(),
                    pos: c.pos as u64,
                    overlap: c.overlap.clone(),
                    optional: opt_fields_to_bin(&c.optional),
                })
                .collect(),
            paths: gfa
                .paths
                .iter()
                .map(|p| BinPath {
                    path_name: p.path_name.clone(),
                    segment_names: p.segment_names.clone(),
                    overlaps: p
                        .overlaps
                        .iter()
                        .map(|o| o.as_ref().map(BinCIGAR::from))
                        .collect(),
                    optional: opt_fields_to_bin(&p.optional),
                })
                .collect(),
            walks: gfa
                .walks
                .iter()
                .map(|w| BinWalk {
                    sample_id: w.sample_id.clone(),
                    hap_index: w.hap_index as u64,
                    seq_id: w.seq_id.clone(),
                    seq_start: w.seq_start.map(|x| x as u64),
                    seq_end: w.seq_end.map(|x| x as u64),
                    segment_path: w
                        .segment_path
                        .iter()
                        .map(|(n, o)| (n.clone(), (*o).into()))
                        .collect(),
                    optional: opt_fields_to_bin(&w.optional),
                })
                .collect(),
            jumps: gfa
                .jumps
                .iter()
                .map(|j| BinJump {
                    from_segment: j.from_segment.clone(),
                    from_orient: j.from_orient.into(),
                    to_segment: j.to_segment.clone(),
                    to_orient: j.to_orient.into(),
                    distance: j.distance,
                    optional: opt_fields_to_bin(&j.optional),
                })
                .collect(),
        }
    }
}

impl From<BinGFA> for GFA<Vec<u8>, Vec<OptField>> {
    fn from(bin: BinGFA) -> Self {
        GFA {
            header: Header {
                version: bin.header.version,
                optional: bin_to_opt_fields(bin.header.optional),
            },
            segments: bin
                .segments
                .into_iter()
                .map(|s| Segment {
                    name: s.name,
                    sequence: s.sequence,
                    optional: bin_to_opt_fields(s.optional),
                })
                .collect(),
            links: bin
                .links
                .into_iter()
                .map(|l| Link {
                    from_segment: l.from_segment,
                    from_orient: l.from_orient.into(),
                    to_segment: l.to_segment,
                    to_orient: l.to_orient.into(),
                    overlap: l.overlap,
                    optional: bin_to_opt_fields(l.optional),
                })
                .collect(),
            containments: bin
                .containments
                .into_iter()
                .map(|c| Containment {
                    container_name: c.container_name,
                    container_orient: c.container_orient.into(),
                    contained_name: c.contained_name,
                    contained_orient: c.contained_orient.into(),
                    pos: c.pos as usize,
                    overlap: c.overlap,
                    optional: bin_to_opt_fields(c.optional),
                })
                .collect(),
            paths: bin
                .paths
                .into_iter()
                .map(|p| {
                    GfaPath::new(
                        p.path_name,
                        p.segment_names,
                        p.overlaps
                            .into_iter()
                            .map(|o| o.map(CIGAR::from))
                            .collect(),
                        bin_to_opt_fields(p.optional),
                    )
                })
                .collect(),
            walks: bin
                .walks
                .into_iter()
                .map(|w| Walk {
                    sample_id: w.sample_id,
                    hap_index: w.hap_index as usize,
                    seq_id: w.seq_id,
                    seq_start: w.seq_start.map(|x| x as usize),
                    seq_end: w.seq_end.map(|x| x as usize),
                    segment_path: w
                        .segment_path
                        .into_iter()
                        .map(|(n, o)| (n, o.into()))
                        .collect(),
                    optional: bin_to_opt_fields(w.optional),
                })
                .collect(),
            jumps: bin
                .jumps
                .into_iter()
                .map(|j| Jump {
                    from_segment: j.from_segment,
                    from_orient: j.from_orient.into(),
                    to_segment: j.to_segment,
                    to_orient: j.to_orient.into(),
                    distance: j.distance,
                    optional: bin_to_opt_fields(j.optional),
                })
                .collect(),
        }
    }
}

/// Serialize a GFA object to a binary format and write to a file.
///
/// # Arguments
/// * `gfa` - The GFA object to serialize
/// * `path` - The path to write the binary file to (typically with `.gfabin` extension)
///
/// # Example
/// ```no_run
/// use gfa::binary_format::write_binary;
/// use gfa::gfa::GFA;
/// use gfa::optfields::OptionalFields;
///
/// let gfa: GFA<Vec<u8>, OptionalFields> = GFA::new();
/// write_binary(&gfa, "output.gfabin").unwrap();
/// ```
pub fn write_binary<T: OptFields, P: AsRef<Path>>(
    gfa: &GFA<Vec<u8>, T>,
    path: P,
) -> Result<(), BinaryFormatError> {
    let file = File::create(path)?;
    let mut writer = BufWriter::new(file);
    write_binary_to_writer(gfa, &mut writer)
}

/// Serialize a GFA object to a binary format and write to a writer.
pub fn write_binary_to_writer<T: OptFields, W: Write>(
    gfa: &GFA<Vec<u8>, T>,
    writer: &mut W,
) -> Result<(), BinaryFormatError> {
    // Write magic and version
    writer.write_all(MAGIC)?;
    writer.write_all(&[VERSION])?;

    // Convert to binary representation and encode
    let bin_gfa = BinGFA::from(gfa);
    let config = bincode::config::standard();
    bincode::encode_into_std_write(&bin_gfa, writer, config)?;

    Ok(())
}

/// Serialize a GFA object to a binary format and return as bytes.
pub fn to_binary<T: OptFields>(
    gfa: &GFA<Vec<u8>, T>,
) -> Result<Vec<u8>, BinaryFormatError> {
    let mut buffer = Vec::new();
    write_binary_to_writer(gfa, &mut buffer)?;
    Ok(buffer)
}

/// Deserialize a GFA object from a binary file.
///
/// # Arguments
/// * `path` - The path to the binary file (typically with `.gfabin` extension)
///
/// # Returns
/// A GFA object with `Vec<u8>` segment names and `Vec<OptField>` optional fields.
///
/// # Example
/// ```no_run
/// use gfa::binary_format::read_binary;
///
/// let gfa = read_binary("input.gfabin").unwrap();
/// println!("Loaded {} segments", gfa.segments.len());
/// ```
pub fn read_binary<P: AsRef<Path>>(
    path: P,
) -> Result<GFA<Vec<u8>, Vec<OptField>>, BinaryFormatError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    read_binary_from_reader(&mut reader)
}

/// Deserialize a GFA object from a reader.
pub fn read_binary_from_reader<R: Read>(
    reader: &mut R,
) -> Result<GFA<Vec<u8>, Vec<OptField>>, BinaryFormatError> {
    // Read and verify magic
    let mut magic = [0u8; 6];
    reader.read_exact(&mut magic)?;
    if &magic != MAGIC {
        return Err(BinaryFormatError::InvalidMagic);
    }

    // Read and verify version
    let mut version = [0u8; 1];
    reader.read_exact(&mut version)?;
    if version[0] != VERSION {
        return Err(BinaryFormatError::UnsupportedVersion(version[0]));
    }

    // Decode the binary GFA
    let config = bincode::config::standard();
    let bin_gfa: BinGFA = bincode::decode_from_std_read(reader, config)?;

    Ok(GFA::from(bin_gfa))
}

/// Deserialize a GFA object from binary bytes.
pub fn from_binary(
    data: &[u8],
) -> Result<GFA<Vec<u8>, Vec<OptField>>, BinaryFormatError> {
    let mut reader = std::io::Cursor::new(data);
    read_binary_from_reader(&mut reader)
}

#[cfg(test)]
mod binary_format_tests {
    use super::*;
    use crate::gfa::Segment;
    use crate::optfields::OptionalFields;

    #[test]
    fn test_roundtrip_empty_gfa() {
        let gfa: GFA<Vec<u8>, OptionalFields> = GFA::new();
        let bytes = to_binary(&gfa).unwrap();
        let restored = from_binary(&bytes).unwrap();

        assert_eq!(gfa.segments.len(), restored.segments.len());
        assert_eq!(gfa.links.len(), restored.links.len());
        assert_eq!(gfa.paths.len(), restored.paths.len());
    }

    #[test]
    fn test_roundtrip_with_segments() {
        let mut gfa: GFA<Vec<u8>, OptionalFields> = GFA::new();
        gfa.segments.push(Segment::new(b"seg1", b"ACGT"));
        gfa.segments.push(Segment::new(b"seg2", b"GGCC"));

        let bytes = to_binary(&gfa).unwrap();
        let restored = from_binary(&bytes).unwrap();

        assert_eq!(gfa.segments.len(), restored.segments.len());
        assert_eq!(gfa.segments[0].name, restored.segments[0].name);
        assert_eq!(gfa.segments[0].sequence, restored.segments[0].sequence);
        assert_eq!(gfa.segments[1].name, restored.segments[1].name);
        assert_eq!(gfa.segments[1].sequence, restored.segments[1].sequence);
    }

    #[test]
    fn test_roundtrip_with_links() {
        let mut gfa: GFA<Vec<u8>, OptionalFields> = GFA::new();
        gfa.segments.push(Segment::new(b"seg1", b"ACGT"));
        gfa.segments.push(Segment::new(b"seg2", b"GGCC"));
        gfa.links.push(Link::new(
            b"seg1",
            Orientation::Forward,
            b"seg2",
            Orientation::Backward,
            b"4M",
        ));

        let bytes = to_binary(&gfa).unwrap();
        let restored = from_binary(&bytes).unwrap();

        assert_eq!(gfa.links.len(), restored.links.len());
        assert_eq!(gfa.links[0].from_segment, restored.links[0].from_segment);
        assert_eq!(gfa.links[0].from_orient, restored.links[0].from_orient);
        assert_eq!(gfa.links[0].to_segment, restored.links[0].to_segment);
        assert_eq!(gfa.links[0].to_orient, restored.links[0].to_orient);
        assert_eq!(gfa.links[0].overlap, restored.links[0].overlap);
    }

    #[test]
    fn test_magic_validation() {
        let bad_data = b"BADMAG\x01";
        let result = from_binary(bad_data);
        assert!(matches!(result, Err(BinaryFormatError::InvalidMagic)));
    }

    #[test]
    fn test_version_validation() {
        let bad_data = b"GFABIN\x99";
        let result = from_binary(bad_data);
        assert!(matches!(
            result,
            Err(BinaryFormatError::UnsupportedVersion(0x99))
        ));
    }
}
