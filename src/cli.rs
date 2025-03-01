use clap::{command, Parser, ValueEnum};
use serde::{Deserialize, Serialize};
use std::{ops::RangeInclusive, path::PathBuf};

use crate::utils::{parse_range, parse_score_filters};

// pub type ScoreFilter = (String, RangeInclusive<usize>, Option<bool>);

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ScoreFilter {
    /// Name of the score filter
    pub name: String,
    /// Range of allowed values
    pub range: RangeInclusive<usize>,
    /// If true, images that do not specify a score values for this filter will be matched. Default is: false
    pub allow_unscored: bool,
}

#[derive(Debug, Parser)]
#[command(name = "kanumi")]
#[command(about = "Select / filter image collections", long_about = None)]
pub struct Cli {
    /// Root directory to use to search for collection of images
    pub directory: Option<PathBuf>,

    /// Restrict output to specified node type
    #[arg(
        short = 't',
        long = "type",
        default_value_t = NodeType::Image,
        value_enum
    )]
    pub node_type: NodeType,

    /// Path to the CSV file containing individual image scores
    #[arg(short, long = "metadata-file")]
    pub metadata_path: Option<PathBuf>,

    /// Only show images with scores that match a specific range
    #[arg(short = 's', long = "scores", value_parser = parse_score_filters)]
    pub score_filters: Option<Vec<ScoreFilter>>,

    /// Only show images with a width contained within this range
    #[arg(short = 'W', long = "width", value_parser = parse_range)]
    pub width_range: Option<RangeInclusive<usize>>,

    /// Only show images with a height contained within this range
    #[arg(short = 'H', long = "height", value_parser = parse_range)]
    pub height_range: Option<RangeInclusive<usize>>,

    /// Generate default configuration
    #[arg(short = 'c', long = "conf-gen", exclusive = true)]
    pub generate_config: bool,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    #[value(name = "directory", alias = "dir", alias = "d")]
    Directory,
    #[value(name = "image", alias = "img", alias = "i")]
    Image,
}
