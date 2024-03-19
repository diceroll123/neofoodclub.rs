use crate::oddschange::OddsChange;
use crate::utils::{convert_from_utc_to_nst, timestamp_to_utc};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RoundData {
    pub foods: Option<[[u8; 10]; 5]>,
    pub round: u16,
    pub start: Option<String>,
    pub pirates: [[u8; 4]; 5],
    pub currentOdds: [[u8; 5]; 5],
    pub openingOdds: [[u8; 5]; 5],
    pub winners: Option<[u8; 5]>,
    pub timestamp: Option<String>,
    pub changes: Option<Vec<OddsChange>>,
    pub lastChange: Option<String>,
}

impl RoundData {
    /// Returns the start time of the round in NST.
    /// If the start time is not available, returns None.
    pub fn start_nst(&self) -> Option<DateTime<Tz>> {
        self.start
            .as_ref()
            .map(|start| convert_from_utc_to_nst(timestamp_to_utc(start)))
    }

    /// Returns the last change time of the round in NST.
    /// If the last change time is not available, returns None.
    pub fn last_change_nst(&self) -> Option<DateTime<Tz>> {
        self.lastChange
            .as_ref()
            .map(|last_change| convert_from_utc_to_nst(timestamp_to_utc(last_change)))
    }

    /// Returns the timestamp of the round in NST.
    /// If the timestamp is not available, returns None.
    pub fn timestamp_nst(&self) -> Option<DateTime<Tz>> {
        self.timestamp
            .as_ref()
            .map(|timestamp| convert_from_utc_to_nst(timestamp_to_utc(timestamp)))
    }

    /// Returns the start time of the round in UTC.
    /// If the start time is not available, returns None.
    pub fn start_utc(&self) -> Option<DateTime<Utc>> {
        self.start.as_ref().map(|start| timestamp_to_utc(start))
    }

    /// Returns the last change time of the round in UTC.
    /// If the last change time is not available, returns None.
    pub fn last_change_utc(&self) -> Option<DateTime<Utc>> {
        self.lastChange
            .as_ref()
            .map(|last_change| timestamp_to_utc(last_change))
    }

    /// Returns the timestamp of the round in UTC.
    /// If the timestamp is not available, returns None.
    pub fn timestamp_utc(&self) -> Option<DateTime<Utc>> {
        self.timestamp
            .as_ref()
            .map(|timestamp| timestamp_to_utc(timestamp))
    }
}
