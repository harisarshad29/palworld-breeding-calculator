use std::collections::{HashSet, VecDeque};

use serde::Serialize;

use crate::{combinations_for_target, find_pal_by_name, AppState};

#[derive(Debug, Clone, Serialize)]
pub struct ChainStep {
    pub parent_a: String,
    pub parent_b: String,
    pub child: String,
    pub method: String,
}

#[derive(Clone)]
struct SearchNode {
    steps: Vec<ChainStep>,
    have: HashSet<String>,
}

fn inventory_key(have: &HashSet<String>) -> Vec<String> {
    let mut names: Vec<_> = have.iter().cloned().collect();
    names.sort();
    names
}

/// Explore executable inventories, never counting a planned child as owned.
/// The bounded search may return None without proving that no route exists.
fn search_chain(
    owned: &str,
    goal: &str,
    max_steps: usize,
    recipes: &[ChainStep],
    additional_owned: &[String],
) -> Option<Vec<ChainStep>> {
    if owned == goal {
        return Some(vec![]);
    }
    let mut have = HashSet::new();
    have.insert(owned.to_string());
    have.extend(additional_owned.iter().cloned());
    if have.contains(goal) {
        return Some(vec![]);
    }
    let mut seen = HashSet::new();
    seen.insert(inventory_key(&have));
    let mut queue = VecDeque::from([SearchNode { steps: vec![], have }]);
    const MAX_STATES: usize = 8_000;

    while let Some(node) = queue.pop_front() {
        if node.steps.len() >= max_steps {
            continue;
        }
        for recipe in recipes {
            if node.have.contains(&recipe.child)
                || !node.have.contains(&recipe.parent_a)
                || !node.have.contains(&recipe.parent_b)
            {
                continue;
            }
            let mut steps = node.steps.clone();
            steps.push(recipe.clone());
            if recipe.child == goal {
                return Some(steps);
            }
            let mut have = node.have.clone();
            have.insert(recipe.child.clone());
            let key = inventory_key(&have);
            if seen.contains(&key) {
                continue;
            }
            if seen.len() >= MAX_STATES {
                return None;
            }
            seen.insert(key);
            queue.push_back(SearchNode { steps, have });
        }
    }
    None
}

pub fn find_breeding_chain(
    state: &AppState,
    owned: &str,
    goal: &str,
    max_steps: usize,
    additional_owned: &[String],
) -> Option<Vec<ChainStep>> {
    let owned_pal = find_pal_by_name(&state.pals, owned)?;
    let goal_pal = find_pal_by_name(&state.pals, goal)?;
    let mut recipes: Vec<_> = state.reverse_combos.iter().flat_map(|(child, pairs)| {
        pairs.iter().map(move |pair| ChainStep {
            parent_a: pair.a.clone(),
            parent_b: pair.b.clone(),
            child: child.clone(),
            method: pair.method.clone(),
        })
    }).collect();
    // HashMap iteration must not decide which equally short route is returned.
    recipes.sort_by(|a, b| (&a.child, &a.parent_a, &a.parent_b).cmp(&(&b.child, &b.parent_a, &b.parent_b)));
    search_chain(&owned_pal.name, &goal_pal.name, max_steps, &recipes, additional_owned)
}

/// Suggest a direct pair, explicitly leaving the unowned partner to acquire.
pub fn direct_parent_combo_for_owned(
    state: &AppState,
    owned: &str,
    goal: &str,
) -> Option<crate::PairResult> {
    let goal_pal = find_pal_by_name(&state.pals, goal)?;
    let mut pairs = combinations_for_target(state, &goal_pal.name);
    pairs.retain(|pair| pair.a.eq_ignore_ascii_case(owned) || pair.b.eq_ignore_ascii_case(owned));
    pairs.sort_by_key(|pair| {
        let partner = if pair.a.eq_ignore_ascii_case(owned) { &pair.b } else { &pair.a };
        let power = find_pal_by_name(&state.pals, partner).map(|pal| pal.power).unwrap_or(0);
        (std::cmp::Reverse(power), partner.clone())
    });
    pairs.into_iter().next()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn recipe(a: &str, b: &str, child: &str) -> ChainStep {
        ChainStep { parent_a: a.into(), parent_b: b.into(), child: child.into(), method: "fixture".into() }
    }

    #[test]
    fn rejects_circular_dependencies_and_missing_partners() {
        let recipes = vec![recipe("Owned", "Goal", "Partner"), recipe("Owned", "Partner", "Goal")];
        assert!(search_chain("Owned", "Goal", 15, &recipes, &[]).is_none());
        assert!(search_chain("Owned", "Goal", 15, &[recipe("Owned", "Missing", "Goal")], &[]).is_none());
    }

    #[test]
    fn returns_steps_only_after_both_parents_are_available() {
        let recipes = vec![recipe("B", "A", "Goal"), recipe("Owned", "Owned", "A"), recipe("A", "Owned", "B")];
        let steps = search_chain("Owned", "Goal", 3, &recipes, &[]).unwrap();
        assert_eq!(steps.iter().map(|s| s.child.as_str()).collect::<Vec<_>>(), vec!["A", "B", "Goal"]);
        let mut have = HashSet::from(["Owned".to_string()]);
        for step in steps {
            assert!(have.contains(&step.parent_a) && have.contains(&step.parent_b));
            have.insert(step.child);
        }
        assert!(search_chain("Owned", "Goal", 2, &recipes, &[]).is_none());
    }

    #[test]
    fn chooses_fewer_steps_without_pruning_parent_candidates() {
        let mut recipes: Vec<_> = (0..10).map(|i| recipe("Owned", "Missing", &format!("Decoy{i}"))).collect();
        recipes.extend([recipe("Owned", "Owned", "A"), recipe("A", "Owned", "Goal"), recipe("Owned", "Owned", "Goal")]);
        assert_eq!(search_chain("Owned", "Goal", 15, &recipes, &[]).unwrap().len(), 1);
    }

    #[test]
    fn owned_goal_needs_no_steps_and_zero_budget_cannot_breed() {
        assert!(search_chain("Owned", "Owned", 0, &[], &[]).unwrap().is_empty());
        assert!(search_chain("Owned", "Goal", 0, &[recipe("Owned", "Owned", "Goal")], &[]).is_none());
    }
}

#[cfg(test)]
mod roster_tests {
    use super::*;

    #[test]
    fn additional_owned_species_unlock_a_real_parent_pair() {
        let recipes = vec![ChainStep {
            parent_a: "Owned".into(), parent_b: "Partner".into(),
            child: "Goal".into(), method: "fixture".into(),
        }];
        assert!(search_chain("Owned", "Goal", 15, &recipes, &[]).is_none());
        assert_eq!(search_chain("Owned", "Goal", 15, &recipes, &["Partner".into()]).unwrap().len(), 1);
        assert!(search_chain("Owned", "Goal", 15, &recipes, &["Goal".into()]).unwrap().is_empty());
    }
}
