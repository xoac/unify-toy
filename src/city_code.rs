use heapless::String;
use serde::{Deserialize, Serialize};
use std::str;
use thiserror::Error;

#[derive(Debug, Clone, Hash, PartialEq, Eq, Error)]
#[error("`city_code` must be valid 3-letters city code")]
pub struct CityCodeError;

/// Valid 3-letters city code
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
pub struct CityCode(String<3>);

impl str::FromStr for CityCode {
    type Err = CityCodeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 3 {
            return Err(CityCodeError);
        }
        let new = String::from(s);
        Ok(Self(new))
    }
}
