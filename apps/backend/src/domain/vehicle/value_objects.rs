use crate::domain::errors::DomainError;

/// A normalised vehicle plate number (trimmed and uppercased).
///
/// # Errors
/// Returns [`DomainError::InvalidPlateNumber`] when the value is empty after trimming.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlateNumber(String);

impl PlateNumber {
    pub fn new(raw: String) -> Result<Self, DomainError> {
        let normalized = raw.trim().to_uppercase();
        if normalized.is_empty() {
            return Err(DomainError::InvalidPlateNumber(raw));
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<PlateNumber> for String {
    fn from(p: PlateNumber) -> Self {
        p.0
    }
}

/// Odometer reading in kilometres. A `u32` guarantees non-negative values by
/// type — no runtime validation is required.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Odometer(u32);

impl Odometer {
    pub fn new(value: u32) -> Self {
        Self(value)
    }

    pub fn value(self) -> u32 {
        self.0
    }
}

/// Fuel type supported by the application.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuelType {
    Petrol,
    Diesel,
    Electric,
    Hybrid,
}

impl FuelType {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Petrol => "petrol",
            Self::Diesel => "diesel",
            Self::Electric => "electric",
            Self::Hybrid => "hybrid",
        }
    }

    /// Parse a fuel type from a lowercase string slice.
    ///
    /// # Errors
    /// Returns [`DomainError::InvalidPlateNumber`] (reused as a generic
    /// validation sentinel) when the value is not one of the known variants.
    pub fn parse(s: &str) -> Result<Self, DomainError> {
        match s {
            "petrol" => Ok(Self::Petrol),
            "diesel" => Ok(Self::Diesel),
            "electric" => Ok(Self::Electric),
            "hybrid" => Ok(Self::Hybrid),
            _ => Err(DomainError::InvalidPlateNumber(format!(
                "unknown fuel type: {s}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plate_number_trimmed_and_uppercased() {
        let p = PlateNumber::new("  b 1234 xyz  ".to_string()).unwrap();
        assert_eq!(p.as_str(), "B 1234 XYZ");
    }

    #[test]
    fn empty_plate_number_rejected() {
        assert!(PlateNumber::new(String::new()).is_err());
    }

    #[test]
    fn whitespace_only_plate_number_rejected() {
        assert!(PlateNumber::new("   ".to_string()).is_err());
    }

    #[test]
    fn odometer_created_from_u32() {
        let o = Odometer::new(50_000);
        assert_eq!(o.value(), 50_000);
    }

    #[test]
    fn odometer_zero_is_valid() {
        let o = Odometer::new(0);
        assert_eq!(o.value(), 0);
    }

    #[test]
    fn fuel_type_parse_petrol() {
        assert_eq!(FuelType::parse("petrol").unwrap(), FuelType::Petrol);
    }

    #[test]
    fn fuel_type_parse_unknown_returns_err() {
        assert!(FuelType::parse("gasoline").is_err());
    }

    #[test]
    fn fuel_type_round_trip() {
        for ft in [
            FuelType::Petrol,
            FuelType::Diesel,
            FuelType::Electric,
            FuelType::Hybrid,
        ] {
            assert_eq!(FuelType::parse(ft.as_str()).unwrap(), ft);
        }
    }
}
