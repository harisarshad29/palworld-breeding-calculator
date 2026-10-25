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

fn read_data(path: &str, bundled: &str) -> String {
    match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => bundled.to_string(),
        Err(error) => panic!("Cannot read {path}: {error}"),
    }
}

fn parse_pals(raw: &str) -> Result<Vec<Pal>, String> {
    let parsed: PalsFile = serde_json::from_str(raw).map_err(|e| format!("Invalid data/pals.json: {e}"))?;
    if parsed.pals.is_empty() {
        return Err("data/pals.json must contain at least one Pal".into());
    }
    let mut names = std::collections::HashSet::new();
    for pal in &parsed.pals {
        if pal.name.trim().is_empty() || pal.power < 0 || !names.insert(pal.name.to_lowercase()) {
            return Err(format!("Invalid or duplicate Pal: {}", pal.name));
        }
    }
    Ok(parsed.pals)
}

pub fn load_pals() -> Vec<Pal> {
    let raw = read_data("data/pals.json", include_str!("../data/pals.json"));
    parse_pals(&raw).unwrap_or_else(|error| panic!("{error}"))
}

pub fn load_pal_locations() -> HashMap<String, PalLocation> {
    let raw = read_data("data/pal_locations.json", include_str!("../data/pal_locations.json"));
    serde_json::from_str::<LocationsFile>(&raw)
        .expect("Invalid data/pal_locations.json")
        .locations
}

fn parse_special_combos(raw: &str) -> Result<HashMap<String, String>, String> {
    let parsed: CombosFile = serde_json::from_str(raw)
        .map_err(|e| format!("Invalid data/special_combos.json: {e}"))?;
    if parsed.combos.is_empty() {
        return Err("data/special_combos.json must not be empty".into());
    }
    let mut combos = HashMap::new();
    for combo in parsed.combos {
        if [&combo.parent_a, &combo.parent_b, &combo.child].iter().any(|name| name.trim().is_empty()) {
            return Err("Special combination contains an empty Pal name".into());
        }
        let key = combo_key(&combo.parent_a, &combo.parent_b);
        if combos.insert(key.clone(), combo.child).is_some() {
            return Err(format!("Duplicate special combination: {key}"));
        }
    }
    Ok(combos)
}

pub fn load_special_combos() -> HashMap<String, String> {
    let raw = read_data("data/special_combos.json", include_str!("../data/special_combos.json"));
    parse_special_combos(&raw).unwrap_or_else(|error| panic!("{error}"))
}

fn combo_key(a: &str, b: &str) -> String {
    if a <= b {
        format!("{a}|{b}")
    } else {
        format!("{b}|{a}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_or_empty_rosters_instead_of_using_demo_data() {
        assert!(parse_pals("not json").is_err());
        assert!(parse_pals(r#"{"pals":[]}"#).is_err());
        assert!(parse_pals(r#"{"pals":[{"name":"A","power":1},{"name":"a","power":2}]}"#).is_err());
        assert!(parse_pals(include_str!("../data/pals.json")).is_ok());
    }

    #[test]
    fn rejects_invalid_empty_or_duplicate_special_combos() {
        assert!(parse_special_combos("not json").is_err());
        assert!(parse_special_combos(r#"{"combos":[]}"#).is_err());
        assert!(parse_special_combos(r#"{"combos":[{"parent_a":"A","parent_b":"B","child":"C"},{"parent_a":"B","parent_b":"A","child":"C"}]}"#).is_err());
        assert!(parse_special_combos(include_str!("../data/special_combos.json")).is_ok());
    }
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct BreedingTable {
    pub pal_names: Vec<String>,
    pub children: Vec<i32>,
    pub gender_outcomes: Vec<TableOutcome>,
}

#[derive(Debug, Clone, Deserialize, serde::Serialize)]
pub struct TableOutcome {
    pub a: usize,
    pub b: usize,
    pub child: usize,
    pub a_gender: String,
    pub b_gender: String,
}

impl BreedingTable {
    pub fn offset(&self, a: usize, b: usize) -> usize {
        let (a, b) = if a <= b { (a, b) } else { (b, a) };
        a * self.pal_names.len() - a * a.saturating_sub(1) / 2 + b - a
    }

    pub fn outcomes(&self, a: usize, b: usize) -> Vec<TableOutcome> {
        let child = self.children[self.offset(a, b)];
        if child >= 0 {
            return vec![TableOutcome { a, b, child: child as usize, a_gender: "wildcard".into(), b_gender: "wildcard".into() }];
        }
        let swapped = a > b;
        let (low, high) = if swapped { (b, a) } else { (a, b) };
        self.gender_outcomes.iter().filter(|row| row.a == low && row.b == high).map(|row| {
            let mut outcome = row.clone();
            if swapped {
                std::mem::swap(&mut outcome.a, &mut outcome.b);
                std::mem::swap(&mut outcome.a_gender, &mut outcome.b_gender);
            }
            outcome
        }).collect()
    }
}

pub fn load_breeding_table(pals: &[Pal]) -> BreedingTable {
    let raw = read_data("data/breeding_table.json", include_str!("../data/breeding_table.json"));
    let table: BreedingTable = serde_json::from_str(&raw).expect("Invalid data/breeding_table.json");
    let n = pals.len();
    assert_eq!(table.pal_names, pals.iter().map(|pal| pal.name.clone()).collect::<Vec<_>>(), "Breeding table and roster must use the same ordered names");
    assert_eq!(table.children.len(), n * (n + 1) / 2, "Incomplete breeding table");
    assert!(table.children.iter().all(|child| *child == -1 || (0..n as i32).contains(child)), "Invalid child index");
    for a in 0..n {
        for b in a..n {
            assert!(!table.outcomes(a, b).is_empty(), "Missing parent pair outcome");
        }
    }
    for row in &table.gender_outcomes {
        assert!(row.a < n && row.b < n && row.child < n && row.a <= row.b, "Invalid gendered outcome index");
        assert!(matches!(row.a_gender.as_str(), "male" | "female") && matches!(row.b_gender.as_str(), "male" | "female") && row.a_gender != row.b_gender, "Invalid required genders");
    }
    table
}
