use clap::{command, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::{ffi::OsString, ops::RangeInclusive, path::PathBuf};

use crate::utils::common::{parse_range, parse_score_filters};

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

#[derive(Debug, Parser)]
#[command(name = "kanumi")]
#[command(about = "Manage collection of images from your terminal", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// View and manage configuration
    #[command(name = "config", alias = "cfg")]
    Configuration {
        #[command(subcommand)]
        command: ConfigurationCommands,
    },
    /// View and manage metadata
    #[command(name = "metadata", alias = "meta")]
    Metadata {
        #[command(subcommand)]
        command: MetadataCommands,
    },
    /// List images that match given selectors
    #[command(name = "list", alias = "ls")]
    List {
        /// Filter based on score range
        #[arg(short = 's', long = "scores", value_parser = parse_score_filters)]
        score_filters: Option<Vec<ScoreFilter>>,

        /// Filter based on width range
        #[arg(short = 'W', long = "width", value_parser = parse_range)]
        width_range: Option<RangeInclusive<usize>>,

        /// Filter based on height range
        #[arg(short = 'H', long = "height", value_parser = parse_range)]
        height_range: Option<RangeInclusive<usize>>,

        /// Filter based on parent directory
        #[arg(short = 'd', long = "directory")]
        base_directory: Option<PathBuf>,

        /// Ignore selectors preset from config
        #[arg(short = 'i', long = "ignore")]
        ignore_config: bool,

        /// Output in JSON
        #[arg(short = 'j', long = "json")]
        use_json_format: bool,
    },
    /// Scan the entire images directory to find missing data
    Scan {
        /// Output in JSON
        #[arg(short = 'j', long = "json")]
        use_json_format: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum ConfigurationCommands {
    /// Print configuration and exit
    Show,
    /// Generate a default configuration file
    #[command(visible_alias = "gen")]
    Generate {
        /// Only print generated configuration. Does not write to file system
        #[arg(short, long)]
        dry_run: bool,
    },
}

#[derive(Debug, Subcommand)]
pub enum MetadataCommands {
    /// Print all metadatas and exit
    Show,
    /// Get the metadata associated to a given image file
    Get {
        image: PathBuf,
    },
    Edit {
        image: PathBuf,
        metadata: OsString,
    },
    /// Generate default metadata for a given image
    #[command(visible_alias = "gen")]
    Generate {
        image: PathBuf,

        /// Only print generated configuration. Does not write to file system
        #[arg(short, long)]
        dry_run: bool,
    },
    // /// Generate metadata file based on configured images directory
    // #[command(visible_alias = "gen-meta")]
    // GenerateMetadata { image: PathBuf },
}
