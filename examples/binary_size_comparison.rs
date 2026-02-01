//! Example comparing GFA text format vs binary format sizes.
//!
//! Run with: cargo run --example binary_size_comparison

use std::fs;

use gfa::binary_format::write_binary;
use gfa::optfields::OptionalFields;
use gfa::parser::GFAParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let input_path = "test/gfas/diatom.gfa";
    let output_path = "diatom.gfabin";

    // Parse the GFA file
    let parser: GFAParser<Vec<u8>, OptionalFields> = GFAParser::new();
    let gfa = parser.parse_file(input_path)?;

    // Write to binary format
    write_binary(&gfa, output_path)?;

    // Get file sizes
    let original_size = fs::metadata(input_path)?.len();
    let binary_size = fs::metadata(output_path)?.len();
    let ratio = original_size as f64 / binary_size as f64;

    println!("Original GFA size: {} bytes ({:.2} KB)", original_size, original_size as f64 / 1024.0);
    println!("Binary size:       {} bytes ({:.2} KB)", binary_size, binary_size as f64 / 1024.0);
    println!("Compression ratio: {:.2}x", ratio);

    // Clean up
    fs::remove_file(output_path)?;

    Ok(())
}
