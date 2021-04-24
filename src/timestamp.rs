use std::fmt;

use crate::utils::get_timestamp;

use chrono::{prelude::*, Local};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct TimeStamp(i64);

impl TimeStamp {
    pub fn now() -> Self {
        Self(get_timestamp())
    }

    pub fn touch(&mut self) {
        self.0 = get_timestamp();
    }

    pub fn get_timestamp(&self) -> i64 {
        self.0
    }
}

impl From<i64> for TimeStamp {
    fn from(time: i64) -> Self {
        Self(time)
    }
}

impl fmt::Display for TimeStamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", Local.timestamp(self.0, 0).to_rfc2822())
    }
}
