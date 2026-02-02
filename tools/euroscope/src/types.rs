use std::collections::HashSet;
use vacs_protocol::vatsim::PositionId;
use vacs_vatsim::coverage::position;

pub trait ParsePosition: Sized {
    type Error;
    fn from_ese_line(line: &str) -> Result<Self, Self::Error>;
}

impl ParsePosition for position::PositionRaw {
    type Error = String;
    fn from_ese_line(line: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = line.split(':').collect();
        if parts.len() < 7 {
            return Err("Invalid format".to_string());
        }

        let Ok(facility_type) = parts[6].parse() else {
            return Err("Invalid facility type".to_string());
        };

        Ok(Self {
            id: PositionId::from(parts[0]),
            frequency: parts[2].to_string(),
            prefixes: HashSet::from([parts[5].to_string()]),
            facility_type,
            profile_id: None,
        })
    }
}
