use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreFilter {
    /// Name of the score filter
    pub name: String,
    /// Range of allowed values
    pub range: RangeInclusive<usize>,
    /// If true, images that do not specify a score values for this filter will be matched. Default is: false
    #[serde(default)]
    pub allow_unscored: bool,
}
