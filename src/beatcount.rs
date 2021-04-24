use std::fmt;

use crate::timestamp::TimeStamp;

use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct BeatCount(pub u64, pub TimeStamp);

impl From<(u64, i64)> for BeatCount {
    fn from(bc: (u64, i64)) -> BeatCount {
        Self(bc.0, TimeStamp::from(bc.1))
    }
}

impl BeatCount {
    pub fn new() -> Self {
        Self(0, TimeStamp::now())
    }

    pub fn get_count(&self) -> u64 {
        self.0
    }

    pub fn get_timestamp(&self) -> i64 {
        self.1.get_timestamp()
    }

    pub fn increment_and_touch(&mut self) {
        self.0 = self.0 + 1;
        self.1.touch();
    }

    pub fn increment(&mut self) {
        self.0 = self.0 + 1;
    }

    pub fn just_touch(&mut self) {
        self.1.touch();
    }
}

impl Default for BeatCount {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for BeatCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "BeatCount({:?}, {})", self.0, self.1)
    }
}
