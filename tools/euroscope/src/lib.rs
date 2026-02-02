mod types;

use crate::types::ParsePosition;
use encoding_rs::WINDOWS_1252;
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use vacs_data_diagnostics::log;
use vacs_vatsim::FacilityType;
use vacs_vatsim::coverage::position::{PositionConfigFile, PositionRaw};

pub fn parse(
    input: &PathBuf,
    output: &PathBuf,
    prefixes: &[String],
    overwrite: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    log::info(format_args!(
        "Parsing EuroScope sectorfile data from {:?} to {:?}",
        input, output
    ));

    if !input.exists() {
        log::error(format_args!("Input file {:?} does not exist", input));
        return Err("Input file does not exist".into());
    }

    if output.exists() {
        if !output.is_dir() {
            log::error(format_args!("Output {:?} is not a directory", output));
            return Err("Output is not a directory".into());
        }
    } else if let Err(err) = std::fs::create_dir_all(output) {
        log::error(format_args!(
            "Failed to create output directory {:?}: {:?}",
            output, err
        ));
        return Err(err.into());
    }

    let output_positions = output.join("positions.toml");
    if output_positions.exists() {
        if overwrite {
            log::warn(format_args!(
                "Overwriting existing positions output file: {:?}",
                output_positions
            ));
        } else {
            log::error(format_args!(
                "Positions output file {:?} already exists",
                output_positions
            ));
            return Err("Positions output file already exists".into());
        }
    }

    let file = match std::fs::File::open(input) {
        Ok(f) => f,
        Err(err) => {
            log::error(format_args!(
                "Failed to open input file {:?}: {:?}",
                input, err
            ));
            return Err(err.into());
        }
    };

    let decoder = DecodeReaderBytesBuilder::new()
        .encoding(Some(WINDOWS_1252))
        .build(file);
    let reader = BufReader::new(decoder);

    let mut positions = Vec::new();
    let mut in_positions_section = false;

    for line in reader.lines() {
        let Ok(line) = line else {
            break;
        };
        let trimmed = line.trim();

        // Empty line or comment
        if trimmed.is_empty() || trimmed.starts_with(";") {
            continue;
        }
        // Start of positions section
        if trimmed == "[POSITIONS]" {
            in_positions_section = true;
            continue;
        }
        // Start of next section after leaving positions section
        if in_positions_section && trimmed.starts_with("[") && trimmed.ends_with("]") {
            break;
        }
        // Ignore positions outside specified prefixes
        if !prefixes.is_empty() && prefixes.iter().all(|p| !trimmed.starts_with(p)) {
            continue;
        }

        if let Ok(position) = PositionRaw::from_ese_line(trimmed) {
            if position.facility_type == FacilityType::Unknown {
                continue;
            }
            positions.push(position);
        }
    }

    positions.sort_by(|a, b| {
        a.facility_type
            .cmp(&b.facility_type)
            .reverse()
            .then_with(|| a.id.cmp(&b.id))
    });

    let serialized_positions = match toml::to_string_pretty(&PositionConfigFile { positions }) {
        Ok(s) => s,
        Err(err) => {
            log::error(format_args!("Failed to serialize positions: {:?}", err));
            return Err(err.into());
        }
    };

    if let Err(err) = std::fs::write(&output_positions, serialized_positions) {
        log::error(format_args!(
            "Failed to write positions output file {:?}: {:?}",
            output_positions, err
        ));
        return Err(err.into());
    }

    log::info(format_args!("Wrote output files to {:?}", output));
    Ok(())
}
