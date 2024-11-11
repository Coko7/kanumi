use std::{ops::RangeInclusive, path::PathBuf};

use anyhow::anyhow;
use clap::{command, Parser, ValueEnum};

#[derive(Debug, Parser)]
#[command(name = "kanumi")]
#[command(about = "Select / filter select image collections", long_about = None)]
pub struct Cli {
    #[arg(short, long = "config-file")]
    pub config_file_path: Option<PathBuf>,

    /// Root directory to use to search for collection of images
    pub directory: Option<PathBuf>,

    #[arg(
        short = 't',
        long = "type",
        default_value_t = NodeType::Image,
        value_enum
    )]
    pub node_type: NodeType,

    /// Restricts output to NUM entries
    #[arg(short = 'n', long = "num")]
    pub select_count: Option<usize>,

    #[arg(short, long = "metadata-file")]
    pub metadata_path: Option<PathBuf>,

    #[arg(short = 's', long = "score", value_parser = parse_range)]
    pub score_range: Option<RangeInclusive<usize>>,

    #[command(flatten)]
    pub verbose: clap_verbosity_flag::Verbosity,
}

#[derive(ValueEnum, Copy, Clone, Debug, PartialEq, Eq)]
pub enum NodeType {
    #[value(name = "directory", alias = "d")]
    Directory,
    #[value(name = "image", alias = "i")]
    Image,
}

fn parse_range(input: &str) -> Result<RangeInclusive<usize>, anyhow::Error> {
    if let Ok(num) = input.parse::<usize>() {
        return Ok(num..=num);
    }

    if !input.contains("..") {
        return Err(anyhow!(
            "Expected number N or range (N..O) but got: `{}`",
            input
        ));
    }

    let parts: Vec<&str> = input.split("..").collect();
    if parts.len() != 2 {
        return Err(anyhow!(
            "Invalid range format, expected X..Y but got: `{}`",
            input
        ));
    }

    let mut formatted_parts = Vec::new();
    for part in parts {
        if part.is_empty() {
            formatted_parts.push(None);
        } else {
            match part.parse::<usize>() {
                Ok(num) => formatted_parts.push(Some(num)),
                Err(e) => return Err(anyhow!("Failed to parse number: `{}`", e)),
            }
        }
    }

    match formatted_parts.as_slice() {
        [None, Some(end)] => Ok(0..=*end),
        [Some(start), None] => Ok(*start..=usize::MAX),
        [Some(start), Some(end)] => Ok(*start..=*end),
        _ => Err(anyhow!("Range should have at least one boundary")),
    }
}
