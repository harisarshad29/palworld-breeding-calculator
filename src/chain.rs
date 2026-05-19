use std::collections::{HashSet, VecDeque};

use serde::Serialize;

use crate::{combinations_for_target, find_pal_by_name, AppState};

#[derive(Clone, Serialize)]
pub struct ChainStep {
    pub parent_a: String,
    pub parent_b: String,
    pub child: String,
    pub method: String,
}

#[derive(Clone)]
struct SearchNode {
    pending: Vec<String>,
    steps: Vec<ChainStep>,
    have: HashSet<String>,
}

fn pal_power(state: &AppState, name: &str) -> i32 {
    find_pal_by_name(&state.pals, name)
        .map(|p| p.power)
        .unwrap_or(9999)
}

fn owns(have: &HashSet<String>, name: &str) -> bool {
    have.iter().any(|h| h.eq_ignore_ascii_case(name))
}

fn pair_score(state: &AppState, have: &HashSet<String>, pair: &crate::PairResult) -> i32 {
    let mut score = pal_power(state, &pair.a) + pal_power(state, &pair.b);
    for parent in [&pair.a, &pair.b] {
        if owns(have, parent) {
            score -= 1_000_000;
        } else {
            score += pal_power(state, parent) * 80;
        }
    }
    score
}

fn push_pending(pending: &mut Vec<String>, have: &HashSet<String>, name: &str) {
    if owns(have, name) {
        return;
    }
    if pending.iter().any(|p| p.eq_ignore_ascii_case(name)) {
        return;
    }
    pending.push(name.to_string());
}

fn search_key(pending: &[String], have: &HashSet<String>) -> String {
    let mut need: Vec<String> = pending.iter().map(|p| p.to_lowercase()).collect();
    need.sort();
    let mut owned: Vec<String> = have.iter().map(|p| p.to_lowercase()).collect();
    owned.sort();
    format!("{}#{}", need.join(","), owned.join(","))
}

/// Shortest backward chain via BFS (prefers pairs using Pals you already own).
pub fn find_breeding_chain(
    state: &AppState,
    owned: &str,
    goal: &str,
    max_steps: usize,
) -> Option<Vec<ChainStep>> {
    let owned_pal = find_pal_by_name(&state.pals, owned)?;
    let goal_pal = find_pal_by_name(&state.pals, goal)?;
    let owned_name = owned_pal.name.clone();
    let goal_name = goal_pal.name.clone();

    if owned_name.eq_ignore_ascii_case(&goal_name) {
        return Some(vec![]);
    }

    let mut have: HashSet<String> = HashSet::new();
    have.insert(owned_name.clone());

    let mut queue = VecDeque::new();
    queue.push_back(SearchNode {
        pending: vec![goal_name.clone()],
        steps: Vec::new(),
        have,
    });

    let max_branch = 8usize;
    let max_nodes = 8_000usize;
    let mut seen_states = HashSet::new();
    let mut visited = 0usize;

    while let Some(node) = queue.pop_front() {
        visited += 1;
        if visited > max_nodes {
            break;
        }
        let key = search_key(&node.pending, &node.have);
        if !seen_states.insert(key) {
            continue;
        }
        if node.steps.len() > max_steps {
            continue;
        }

        if node.pending.is_empty() {
            if owns(&node.have, &goal_name) {
                return Some(node.steps);
            }
            continue;
        }

        let mut pending = node.pending;
        let target = pending.pop().expect("non-empty pending");

        if owns(&node.have, &target) {
            queue.push_back(SearchNode {
                pending,
                steps: node.steps,
                have: node.have,
            });
            continue;
        }

        let mut pairs = combinations_for_target(state, &target);
        if pairs.is_empty() {
            continue;
        }
        pairs.sort_by(|a, b| pair_score(state, &node.have, a).cmp(&pair_score(state, &node.have, b)));

        for pair in pairs.into_iter().take(max_branch) {
            let mut steps = node.steps.clone();
            steps.push(ChainStep {
                parent_a: pair.a.clone(),
                parent_b: pair.b.clone(),
                child: target.clone(),
                method: pair.method.clone(),
            });

            let mut have = node.have.clone();
            have.insert(target.clone());

            let mut next_pending = pending.clone();
            push_pending(&mut next_pending, &have, &pair.a);
            push_pending(&mut next_pending, &have, &pair.b);

            queue.push_back(SearchNode {
                pending: next_pending,
                steps,
                have,
            });
        }
    }

    None
}

/// When full chain fails but owned Pal is a direct parent for the goal.
pub fn direct_parent_combo_for_owned(
    state: &AppState,
    owned: &str,
    goal: &str,
) -> Option<crate::PairResult> {
    let goal_pal = find_pal_by_name(&state.pals, goal)?;
    let mut pairs = combinations_for_target(state, &goal_pal.name);
    pairs.sort_by(|a, b| pair_score(state, &HashSet::new(), a).cmp(&pair_score(state, &HashSet::new(), b)));
    pairs.into_iter().find(|pair| {
        pair.a.eq_ignore_ascii_case(owned) || pair.b.eq_ignore_ascii_case(owned)
    })
}
