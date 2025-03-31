#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! imx = "0.1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! serde_json = "1.0"
//! anyhow = "1.0"
//! ```

use anyhow::{Context, Result};
use imx::{LabelAlignment, PlotConfig, create_plot};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    images: Vec<String>,
    output: String,
    rows: usize,
    row_labels: Vec<String>,
    column_labels: Vec<String>,
    #[serde(default = "default_alignment")]
    column_label_alignment: String,
    #[serde(default = "default_alignment")]
    row_label_alignment: String,
    #[serde(default = "default_top_padding")]
    top_padding: i32,
    #[serde(default = "default_left_padding")]
    left_padding: i32,
    #[serde(default = "default_font_size")]
    font_size: f32,
    #[serde(default)]
    debug_mode: bool,
}

fn default_alignment() -> String {
    "center".to_string()
}

fn default_top_padding() -> i32 {
    60
}

fn default_left_padding() -> i32 {
    60
}

fn default_font_size() -> f32 {
    40.0
}

fn string_to_alignment(s: &str) -> LabelAlignment {
    match s.to_lowercase().as_str() {
        "start" => LabelAlignment::Start,
        "end" => LabelAlignment::End,
        _ => LabelAlignment::Center,
    }
}

fn main() -> Result<()> {
    // Get JSON config file path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: imx-plot <config.json>");
        std::process::exit(1);
    }
    let config_path = &args[1];

    // Read and parse JSON config file
    let config_content = fs::read_to_string(config_path)
        .with_context(|| format!("Failed to read config file: {}", config_path))?;

    let config: ConfigFile = serde_json::from_str(&config_content)
        .with_context(|| format!("Failed to parse JSON from config file: {}", config_path))?;

    // Convert string paths to PathBuf
    let images: Vec<PathBuf> = config.images.iter().map(|s| PathBuf::from(s)).collect();

    // Convert types for PlotConfig
    let rows_u32 = u32::try_from(config.rows)
        .with_context(|| format!("Failed to convert rows from usize to u32: {}", config.rows))?;

    // Ensure padding values are non-negative before converting to u32
    let top_padding_u32 = if config.top_padding < 0 {
        println!(
            "Warning: negative top_padding value ({}), using 0 instead",
            config.top_padding
        );
        0u32
    } else {
        config.top_padding as u32
    };

    let left_padding_u32 = if config.left_padding < 0 {
        println!(
            "Warning: negative left_padding value ({}), using 0 instead",
            config.left_padding
        );
        0u32
    } else {
        config.left_padding as u32
    };

    // Create plot config
    let plot_config = PlotConfig {
        images,
        output: PathBuf::from(&config.output),
        rows: rows_u32,
        row_labels: config.row_labels,
        column_labels: config.column_labels,
        column_label_alignment: string_to_alignment(&config.column_label_alignment),
        row_label_alignment: string_to_alignment(&config.row_label_alignment),
        top_padding: top_padding_u32,
        left_padding: left_padding_u32,
        font_size: Some(config.font_size),
        debug_mode: config.debug_mode,
    };

    // Create the plot
    create_plot(&plot_config).with_context(|| "Failed to create plot")?;

    println!("Successfully created plot at: {}", config.output);
    if config.debug_mode {
        let debug_output = if let Some(ext) = PathBuf::from(&config.output).extension() {
            let mut output_path = PathBuf::from(&config.output);
            output_path.set_file_name(format!(
                "{}_debug.{}",
                output_path.file_stem().unwrap().to_string_lossy(),
                ext.to_string_lossy()
            ));
            output_path.to_string_lossy().to_string()
        } else {
            format!("{}_debug", config.output)
        };
        println!("Debug visualization saved at: {}", debug_output);
    }

    Ok(())
}
