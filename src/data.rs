use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use super::Pal;

#[derive(Deserialize)]
struct PalsFile {
    pals: Vec<Pal>,
}

#[derive(Deserialize)]
struct ComboEntry {
    parent_a: String,
    parent_b: String,
    child: String,
}

#[derive(Deserialize)]
struct CombosFile {
    combos: Vec<ComboEntry>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct PalLocation {
    pub area: String,
    pub coords: String,
}

#[derive(Deserialize)]
struct LocationsFile {
    locations: HashMap<String, PalLocation>,
}

pub fn load_pals() -> Vec<Pal> {
    let raw = fs::read_to_string("data/pals.json").unwrap_or_else(|_| include_str!("../data/pals.json").to_string());
    serde_json::from_str::<PalsFile>(&raw)
        .map(|f| f.pals)
        .unwrap_or_else(|_| fallback_pals())
}

pub fn load_pal_locations() -> HashMap<String, PalLocation> {
    let raw = fs::read_to_string("data/pal_locations.json").unwrap_or_else(|_| {
        include_str!("../data/pal_locations.json").to_string()
    });
    serde_json::from_str::<LocationsFile>(&raw)
        .map(|f| f.locations)
        .unwrap_or_default()
}

pub fn load_special_combos() -> HashMap<String, String> {
    let raw = fs::read_to_string("data/special_combos.json")
        .unwrap_or_else(|_| include_str!("../data/special_combos.json").to_string());
    let parsed: CombosFile = serde_json::from_str(&raw).unwrap_or(CombosFile { combos: vec![] });
    parsed
        .combos
        .into_iter()
        .map(|c| (combo_key(&c.parent_a, &c.parent_b), c.child))
        .collect()
}

fn combo_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

fn fallback_pals() -> Vec<Pal> {
    vec![
        Pal { name: "Lamball".to_string(), power: 1470 },
        Pal { name: "Anubis".to_string(), power: 570 },
        Pal { name: "Jetragon".to_string(), power: 90 },
    ]
}
