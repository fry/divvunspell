
use crate::speller::SpellerConfig;
use serde_derive::{Deserialize, Serialize};
use std::borrow::Cow;
use treediff::Value;
use structdiff::Diff;
use structdiff_derive::Diff;

use crate::speller::SpellerConfigChangeset;

#[derive(Debug, Default, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Diff)]
pub struct Time {
    pub secs: u64,
    pub subsec_nanos: u32,
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let ms = self.secs * 1000 + (self.subsec_nanos as u64 / 1000000);
        write!(f, "{}ms", ms)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Diff)]
pub struct Suggestion {
    pub value: String,
    pub weight: f32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Diff)]
pub struct Report {
    pub metadata: crate::archive::meta::SpellerMetadata,
    pub config: SpellerConfig,
    pub summary: Summary,
    pub results: Vec<AccuracyResult>,
    pub start_timestamp: Time,
    pub total_time: Time,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Diff)]
pub struct AccuracyResult {
    pub input: String,
    pub expected: String,
    pub suggestions: Vec<Suggestion>,
    pub position: Option<usize>,
    pub time: Time,
}

#[derive(Serialize, Deserialize, PartialEq, Default, Debug, Clone, Diff)]
pub struct Summary {
    pub total_words: u32,
    pub first_position: u32,
    pub top_five: u32,
    pub any_position: u32,
    pub no_suggestions: u32,
    pub only_wrong: u32,
    pub slowest_lookup: Time,
    pub fastest_lookup: Time,
    pub average_time: Time,
    pub average_time_95pc: Time,
}

impl std::fmt::Display for Summary {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        let percent =
            |v: u32| -> String { format!("{:.2}%", v as f32 / self.total_words as f32 * 100f32) };

        write!(
            f,
            "[#1] {} [^5] {} [any] {} [none] {} [wrong] {} [fast] {} [slow] {}",
            percent(self.first_position),
            percent(self.top_five),
            percent(self.any_position),
            percent(self.no_suggestions),
            percent(self.only_wrong),
            self.fastest_lookup,
            self.slowest_lookup
        )
    }
}

impl Summary {
    pub fn new(results: &[AccuracyResult]) -> Summary {
        let mut summary = Summary::default();

        results.iter().for_each(|result| {
            summary.total_words += 1;

            if let Some(position) = result.position {
                summary.any_position += 1;

                if position == 0 {
                    summary.first_position += 1;
                }

                if position < 5 {
                    summary.top_five += 1;
                }
            } else if result.suggestions.len() == 0 {
                summary.no_suggestions += 1;
            } else {
                summary.only_wrong += 1;
            }
        });

        summary.slowest_lookup = results
            .iter()
            .max_by(|x, y| x.time.cmp(&y.time))
            .unwrap()
            .time
            .clone();
        summary.fastest_lookup = results
            .iter()
            .min_by(|x, y| x.time.cmp(&y.time))
            .unwrap()
            .time
            .clone();

        summary
    }
}
