//use crate::gfa::orientation::*;
//use crate::gfa::traits::*;
use crate::gfa::*;

use crate::{cigar::CIGAR, optfields::*};
use std::fmt::Write;

use std::io::Read as IoRead;
use std::io::Write as IoWrite;
use tempfile::NamedTempFile;

#[test]
fn path_iter() {
    use Orientation::*;

    let cigars = vec![b"4M", b"5M"]
        .iter()
        .map(|bs| CIGAR::from_bytestring(&bs[..]))
        .collect();

    let path: Path<Vec<u8>, _> =
        Path::new("14".into(), "11+,12-,13+".into(), cigars, ());

    let mut path_iter = path.iter();
    assert_eq!(Some(("11".into(), Forward)), path_iter.next());
    assert_eq!(Some(("12".into(), Backward)), path_iter.next());
    assert_eq!(Some(("13".into(), Forward)), path_iter.next());
    assert_eq!(None, path_iter.next());
}

#[test]
fn gfa_line_ref_iter() {
    let parser: crate::parser::GFAParser<usize, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/lil.gfa").unwrap();
    let gfa_lineref = gfa.lines_iter();

    for line in gfa_lineref {
        let seg = line.some_segment();
        println!("{:?}", seg);
    }
}

#[test]
fn gfa_rgfa1_ref_iter() {
    let parser: crate::parser::GFAParser<usize, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/rGFA1_test.gfa").unwrap();
    let gfa_lineref = gfa.lines_iter();

    for line in gfa_lineref {
        let seg = line.some_segment();
        println!("{:?}", seg);
    }
}


#[test]
fn gfa_testgraph_walk_iter() {
    let parser: crate::parser::GFAParser<usize, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/testGraph_1.1.gfa").unwrap();
    let gfa_lineref = gfa.lines_iter();

    for line in gfa_lineref {
        let seg = line.some_segment();
        println!("{:?}", seg);
    }
}


#[test]
fn gfa_testgraph_compact_nopw_iter() {
    let parser: crate::parser::GFAParser<usize, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/testGraph_compact_nopw.gfa").unwrap();
    let gfa_lineref = gfa.lines_iter();

    for line in gfa_lineref {
        let seg = line.some_segment();
        println!("{:?}", seg);
    }
}


#[test]
fn gfa_testgraph_complex_iter() {
    let parser: crate::parser::GFAParser<usize, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/testGraph_complex.gfa").unwrap();
    let gfa_lineref = gfa.lines_iter();

    for line in gfa_lineref {
        let seg = line.some_segment();
        println!("{:?}", seg);
    }
}


#[test]
fn gfa_testgraph_nonnum_iter() {
    let parser: crate::parser::GFAParser<Vec<u8>, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/testGraph_non-num.gfa").unwrap();
    let gfa_lineref = gfa.lines_iter();

    for line in gfa_lineref {
        let seg = line.some_segment();
        println!("{:?}", seg);
    }
}

/*#[test]
fn gfa_jump_read_test() {
    let parser: crate::parser::GFAParser<usize, ()> =
        crate::parser::GFAParser::new();
    let gfa = parser.parse_file(&"./test/gfas/jump.gfa").unwrap();
    //let gfa_lineref = gfa.lines_iter();
    //for line in gfa_lineref {
    //    let seg = line.some_segment();
    //    println!("{:?}", seg);
    //}
}*/

#[test]
fn write_segment_to_string_buffer() {
    use OptFieldVal::*;
    let mut segment: Segment<Vec<u8>, OptionalFields> =
        Segment::new(b"seg1", b"GCCCTA");
    let opt_ij = OptField::new(b"IJ", A(b'x'));
    let opt_ab = OptField::new(b"AB", BInt(vec![1, 2, 3, 52124]));
    let opt_ur = OptField::new(b"UR", Z(Vec::<u8>::from("http://test.com/")));
    let opt_rc = OptField::new(b"RC", Int(123));
    segment.optional = vec![opt_rc, opt_ur, opt_ij, opt_ab];
    let expected = "S\tseg1\tGCCCTA\tRC:i:123\tUR:Z:http://test.com/\tIJ:A:x\tAB:B:I1,2,3,52124";
    let mut string = String::new();
    write!(&mut string, "{}", segment).expect("Error writing to string buffer");
    assert_eq!(string, expected);
}

#[test]
fn write_link_to_string_buffer() {
    let link: Link<Vec<u8>, ()> = Link::new(
        b"13",
        Orientation::Forward,
        b"552",
        Orientation::Backward,
        b"0M",
    );
    let mut string = String::new();
    write!(&mut string, "{}", link).expect("Error writing to string buffer");
    assert_eq!(string, "L\t13\t+\t552\t-\t0M");
}

#[test]
fn write_path_to_string_buffer() {
    use crate::cigar::CIGAR;

    let cigars = vec![b"8M", b"1M", b"3M"]
        .iter()
        .map(|bs| CIGAR::from_bytestring(&bs[..]))
        .collect();

    let path: Path<Vec<u8>, _> =
        Path::new("path1".into(), "13+,51-,241+".into(), cigars, ());

    let mut string = String::new();
    write!(&mut string, "{}", path).expect("Error writing to string buffer");
    assert_eq!(string, "P\tpath1\t13+,51-,241+\t8M,1M,3M");
}

#[test]
fn write_walk_to_string_buffer() {
    use OptFieldVal::*;

    let mut walk: Walk<Vec<u8>, OptionalFields> = Walk::new(
        b"sample1".to_vec(),
        1,
        b"chr1".to_vec(),
        Some(10),
        Some(100),
        vec![
            (b"seg1".to_vec(), Orientation::Forward),
            (b"seg2".to_vec(), Orientation::Backward),
            (b"seg3".to_vec(), Orientation::Forward),
        ],
        vec![],
    );

    let opt_rc = OptField::new(b"RC", Int(123));
    let opt_ij = OptField::new(b"IJ", A(b'x'));
    let opt_ab = OptField::new(b"AB", BInt(vec![1, 2, 3, 52124]));
    let opt_ur = OptField::new(b"UR", Z(Vec::<u8>::from("http://test.com/")));
    walk.optional = vec![opt_rc, opt_ur, opt_ij, opt_ab];

    let expected = "W\tsample1\t1\tchr1\t10\t100\t>seg1<seg2>seg3\tRC:i:123\tUR:Z:http://test.com/\tIJ:A:x\tAB:B:I1,2,3,52124";

    let mut string = String::new();
    write!(&mut string, "{}", walk).expect("Error writing to string buffer");

    assert_eq!(string, expected);
}

#[test]
fn write_gfa_to_string_buffer() {
    use std::io::Read;
    use std::path::PathBuf;

    let parser = crate::parser::GFAParser::new();
    let in_gfa: GFA<Vec<u8>, ()> =
        parser.parse_file(&"./test/gfas/lil.gfa").unwrap();

    let mut file =
        std::fs::File::open(&PathBuf::from("./test/gfas/lil.gfa")).unwrap();
    let mut file_string = String::new();
    file.read_to_string(&mut file_string).unwrap();

    let mut string = String::new();
    write!(&mut string, "{}", in_gfa).expect("Error writing to string buffer");

    assert_eq!(string, file_string);
}

#[test]
fn write_segment_to_file_buffer() {
    use OptFieldVal::*;
    let mut segment: Segment<Vec<u8>, OptionalFields> =
        Segment::new(b"seg1", b"GCCCTA");
    let opt_ij = OptField::new(b"IJ", A(b'x'));
    let opt_ab = OptField::new(b"AB", BInt(vec![1, 2, 3, 52124]));
    let opt_ur = OptField::new(b"UR", Z(Vec::<u8>::from("http://test.com/")));
    let opt_rc = OptField::new(b"RC", Int(123));
    segment.optional = vec![opt_rc, opt_ur, opt_ij, opt_ab];
    let expected = "S\tseg1\tGCCCTA\tRC:i:123\tUR:Z:http://test.com/\tIJ:A:x\tAB:B:I1,2,3,52124";

    let mut tempfile = NamedTempFile::new().expect("Error creating temp file");
    tempfile
        .write_all(format!("{}", segment).as_bytes())
        .expect("Error writing to file buffer.");

    let mut tempfile_reader =
        tempfile.reopen().expect("error re-opening temp file.");

    let mut string = String::new();
    tempfile_reader
        .read_to_string(&mut string)
        .expect("Error parsing file");
    assert_eq!(string, expected);
}

#[test]
fn write_link_to_file_buffer() {
    let link: Link<Vec<u8>, ()> = Link::new(
        b"13",
        Orientation::Forward,
        b"552",
        Orientation::Backward,
        b"0M",
    );
    let mut tempfile = NamedTempFile::new().expect("Error creating temp file");
    tempfile
        .write_all(format!("{}", link).as_bytes())
        .expect("Error writing to file buffer");

    let mut tempfile_reader =
        tempfile.reopen().expect("error re-opening temp file.");

    let mut string = String::new();
    tempfile_reader
        .read_to_string(&mut string)
        .expect("Error parsing file");
    assert_eq!(string, "L\t13\t+\t552\t-\t0M");
}

#[test]
fn write_path_to_file_buffer() {
    use crate::cigar::CIGAR;

    let cigars = vec![b"8M", b"1M", b"3M"]
        .iter()
        .map(|bs| CIGAR::from_bytestring(&bs[..]))
        .collect();

    let path: Path<Vec<u8>, _> =
        Path::new("path1".into(), "13+,51-,241+".into(), cigars, ());

    let mut tempfile = NamedTempFile::new().expect("Error creating temp file");

    tempfile
        .write_all(format!("{}", path).as_bytes())
        .expect("Error writing to file buffer");

    let mut tempfile_reader =
        tempfile.reopen().expect("error re-opening temp file.");

    let mut string = String::new();
    tempfile_reader
        .read_to_string(&mut string)
        .expect("Error parsing file");

    assert_eq!(string, "P\tpath1\t13+,51-,241+\t8M,1M,3M");
}

#[test]
fn write_walk_to_file_buffer() {
    let walk: Walk<Vec<u8>, ()> = Walk::new(
        b"sample1".to_vec(),
        1,
        b"chr1".to_vec(),
        Some(10),
        Some(100),
        vec![
            (b"seg1".to_vec(), Orientation::Forward),
            (b"seg2".to_vec(), Orientation::Backward),
        ],
        (),
    );

    let expected = "W\tsample1\t1\tchr1\t10\t100\t>seg1<seg2";

    let mut tempfile = NamedTempFile::new().expect("Error creating temp file");

    tempfile
        .write_all(format!("{}", walk).as_bytes())
        .expect("Error writing to file buffer");

    let mut tempfile_reader =
        tempfile.reopen().expect("error re-opening temp file.");

    let mut string = String::new();
    tempfile_reader
        .read_to_string(&mut string)
        .expect("Error parsing file");

    assert_eq!(string, expected);
}

#[test]
fn write_gfa_to_file_buffer() {
    use std::path::PathBuf;

    let parser = crate::parser::GFAParser::new();
    let in_gfa: GFA<Vec<u8>, ()> =
        parser.parse_file(&"./test/gfas/lil.gfa").unwrap();

    let mut file =
        std::fs::File::open(&PathBuf::from("./test/gfas/lil.gfa")).unwrap();
    let mut file_string = String::new();
    file.read_to_string(&mut file_string).unwrap();

    let mut tempfile = NamedTempFile::new().expect("Error creating temp file");
    write!(&mut tempfile, "{}", in_gfa).expect("Error writing to file buffer");

    let mut tempfile_reader =
        tempfile.reopen().expect("error re-opening temp file.");

    let mut string = String::new();
    tempfile_reader
        .read_to_string(&mut string)
        .expect("Error parsing file");

    assert_eq!(string, file_string);
}
