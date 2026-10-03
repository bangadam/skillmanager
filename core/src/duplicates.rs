use crate::{markdown, SkillState};
use serde::Serialize;
use std::io;

// ponytail: a fixed 0.8 threshold is enough for a read-only radar; expose tuning if real skills need it.
const NEAR_MATCH_THRESHOLD: f64 = 0.8;

#[derive(Clone, Debug, Serialize)]
pub struct DuplicatePair {
    pub a: SkillState,
    pub b: SkillState,
    pub similarity: f64,
    pub is_exact: bool,
}

struct Text {
    normalized: String,
    shingles: Vec<String>,
}

impl Text {
    fn new(content: &str) -> Self {
        let mut normalized = String::with_capacity(content.len());
        for word in content.split_whitespace() {
            if !normalized.is_empty() { normalized.push(' '); }
            for ch in word.chars().flat_map(char::to_lowercase) { normalized.push(ch); }
        }
        let words: Vec<_> = normalized.split_whitespace().collect();
        let mut shingles: Vec<String> = if words.len() < 2 {
            words.iter().map(|word| (*word).to_owned()).collect()
        } else {
            words.windows(2).map(|pair| pair.join(" ")).collect()
        };
        shingles.sort_unstable();
        Self { normalized, shingles }
    }

    fn similarity(&self, other: &Self) -> f64 {
        if self.normalized == other.normalized { return 1.0; }
        if self.shingles.is_empty() || other.shingles.is_empty() { return 0.0; }
        let (mut i, mut j, mut shared) = (0, 0, 0);
        while i < self.shingles.len() && j < other.shingles.len() {
            match self.shingles[i].cmp(&other.shingles[j]) {
                std::cmp::Ordering::Less => i += 1,
                std::cmp::Ordering::Greater => j += 1,
                std::cmp::Ordering::Equal => { i += 1; j += 1; shared += 1; }
            }
        }
        2.0 * f64::from(shared) / (self.shingles.len() + other.shingles.len()) as f64
    }
}

pub fn find_duplicates(skills: &[SkillState]) -> io::Result<Vec<DuplicatePair>> {
    let texts: Vec<Vec<Text>> = skills.iter().map(|skill| {
        skill.presences.iter().map(|copy| markdown(&copy.path).map(|content| Text::new(content.as_deref().unwrap_or_default()))).collect()
    }).collect::<io::Result<_>>()?;
    let mut pairs = Vec::new();
    for (i, a) in skills.iter().enumerate() {
        for (j, b) in skills.iter().enumerate().skip(i + 1) {
            let (similarity, is_exact) = texts[i].iter().flat_map(|left| texts[j].iter().map(move |right| (left, right)))
                .fold((0.0_f64, false), |(score, exact), (left, right)| {
                    (score.max(left.similarity(right)), exact || left.normalized == right.normalized)
                });
            if similarity >= NEAR_MATCH_THRESHOLD {
                pairs.push(DuplicatePair { a: a.clone(), b: b.clone(), similarity, is_exact });
            }
        }
    }
    Ok(pairs)
}
