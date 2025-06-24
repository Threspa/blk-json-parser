#![cfg_attr(windows, windows_subsystem = "windows")]use anyhow::{anyhow, Result};
use fltk::{
    app, button,
    dialog::{self, NativeFileChooser, NativeFileChooserType},
    enums::{Color, FrameType},
    frame,
    group::Pack,
    prelude::*,
    window::Window,
};
use regex::Regex;
use serde::Serialize;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
};

#[derive(Serialize)]
struct Point {
    x: f64,
    y: f64,
}

#[derive(Serialize)]
#[serde(untagged)]
enum Shape {
    Line {
        name: String,
        #[serde(rename = "type")]
        typ: String,
        start: Point,
        end: Point,
        selected: bool,
    },
    Quad {
        name: String,
        #[serde(rename = "type")]
        typ: String,
        pos1: Point,
        pos2: Point,
        pos3: Point,
        pos4: Point,
        selected: bool,
    },
}

fn extract_block(text: &str, block_name: &str) -> String {
    let pattern = format!(r"(?m){}[\s\n]*\{{", regex::escape(block_name));
    let re = Regex::new(&pattern).unwrap();
    
    if let Some(mat) = re.find(text) {
        let start = mat.end();
        let mut depth = 1;
        let chars: Vec<char> = text.chars().collect();
        let mut i = start;
        
        while i < chars.len() {
            match chars[i] {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        return text[start..i].to_string();
                    }
                }
                _ => {}
            }
            i += 1;
        }
    }
    String::new()
}

fn parse_input(text: &str) -> Result<BTreeMap<String, Shape>> {
    let mut result = BTreeMap::new();
    let mut idx = 0;

    let lines_block = extract_block(text, "drawLines");
    let quads_block = extract_block(text, "drawQuads");
    let combined_text = format!("{}\n{}", lines_block, quads_block);

    // Parse lines
    let line_re = Regex::new(r"(?i)line\s*\{line:p4=([^;]+);move:b=(true|false);\}").unwrap();
    for cap in line_re.captures_iter(&combined_text) {
        let coords_str = cap[1].trim();
        let coords: Vec<f64> = coords_str
            .split(',')
            .map(|s| s.trim().parse().unwrap())
            .collect();

        if coords.len() != 4 {
            return Err(anyhow!("Invalid line coordinates: {}", coords_str));
        }

        result.insert(
            idx.to_string(),
            Shape::Line {
                name: format!("Линия{idx}"),
                typ: "line".to_string(),
                start: Point {
                    x: coords[0],
                    y: coords[1],
                },
                end: Point {
                    x: coords[2],
                    y: coords[3],
                },
                selected: false,
            },
        );
        idx += 1;
    }

    // Parse quads
    let quad_re = Regex::new(
        r"(?i)quad\s*\{tl:p2\s*=\s*([^;]+);\s*tr:p2\s*=\s*([^;]+);\s*br:p2\s*=\s*([^;]+);\s*bl:p2\s*=\s*([^;]+);\}",
    )
    .unwrap();

    for cap in quad_re.captures_iter(&combined_text) {
        let points = (1..=4)
            .map(|i| {
                cap[i]
                    .split(',')
                    .map(|s| s.trim().parse().unwrap())
                    .collect::<Vec<f64>>()
            })
            .collect::<Vec<_>>();

        if points.iter().any(|p| p.len() != 2) {
            return Err(anyhow!("Invalid quad coordinates"));
        }

        result.insert(
            idx.to_string(),
            Shape::Quad {
                name: format!("Четырёхугольник{idx}"),
                typ: "quad".to_string(),
                pos1: Point {
                    x: points[0][0],
                    y: points[0][1],
                },
                pos2: Point {
                    x: points[1][0],
                    y: points[1][1],
                },
                pos3: Point {
                    x: points[2][0],
                    y: points[2][1],
                },
                pos4: Point {
                    x: points[3][0],
                    y: points[3][1],
                },
                selected: false,
            },
        );
        idx += 1;
    }

    Ok(result)
}

fn convert_file() -> Result<()> {
    let mut dialog = NativeFileChooser::new(NativeFileChooserType::BrowseFile);
    dialog.set_filter("BLK and Text files\t*.{blk,txt}");
    dialog.show();

    let path = dialog.filename();
    if path.to_string_lossy().is_empty() {
        return Ok(());
    }

    let content = fs::read_to_string(&path)?;
    let data = parse_input(&content)?;

    let downloads_dir = dirs::download_dir().ok_or_else(|| anyhow!("Couldn't find downloads directory"))?;
    let filename = Path::new(&path)
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow!("Invalid filename"))?;
    let output_path = downloads_dir.join(format!("{}.json", filename));

    fs::write(&output_path, serde_json::to_string_pretty(&data)?)?;

    dialog::alert(
        300,
        200,
        &format!(
            "DONE!\nCHECK IT IN DOWNLOADS:\n{}",
            output_path.file_name().unwrap().to_string_lossy()
        ),
    );

    Ok(())
}

fn main() {
    let app = app::App::default();
    let mut win = Window::default()
        .with_size(300, 200)
        .with_label("BLK to JSON");
    win.set_color(Color::White);

    let mut pack = Pack::default()
        .with_size(200, 150)
        .center_of_parent();
    pack.set_spacing(10);

    let mut label = frame::Frame::default()
        .with_size(0, 40)
        .with_label("BLK to JSON");
    label.set_label_size(25);
    label.set_frame(FrameType::NoBox);

    let mut button = button::Button::default()
        .with_size(0, 60)
        .with_label("CONVERT");
    button.set_color(Color::Black);
    button.set_label_color(Color::White);
    button.set_label_size(14);

    pack.end();
    win.end();
    win.show();

    button.set_callback(|_| {
        if let Err(e) = convert_file() {
            dialog::alert(300, 200, &format!("Error: {}", e));
        }
    });

    app.run().unwrap();
}