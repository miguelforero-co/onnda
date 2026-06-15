//! Auto-learn from corrections (Phase 3).
//!
//! When the user edits a transcription in the history, we diff the original
//! ASR output against the corrected text at the word level and tally each
//! substitution. Once the same correction is seen `PROMOTE_THRESHOLD` times it
//! is promoted into a deterministic replacement rule, so future transcriptions
//! get it right automatically — for both engines.

use crate::settings::{AppSettings, LearnedCorrection, Replacement};

/// How many times a correction must be observed before it becomes a rule.
const PROMOTE_THRESHOLD: u32 = 2;
/// Don't learn rewrites longer than this many words per side (those are edits,
/// not vocabulary corrections).
const MAX_WORDS_PER_SIDE: usize = 4;

fn flush<'a>(
    del: &mut Vec<&'a str>,
    ins: &mut Vec<&'a str>,
    pairs: &mut Vec<(String, String)>,
) {
    if !del.is_empty()
        && !ins.is_empty()
        && del.len() <= MAX_WORDS_PER_SIDE
        && ins.len() <= MAX_WORDS_PER_SIDE
    {
        let from = del.join(" ");
        let to = ins.join(" ");
        if from != to {
            pairs.push((from, to));
        }
    }
    del.clear();
    ins.clear();
}

/// Word-level substitutions between `old` and `new` via an LCS alignment.
/// Returns (from, to) pairs where a run of original words was replaced by a run
/// of corrected words (pure insertions/deletions are ignored — not corrections).
pub fn word_diff(old: &str, new: &str) -> Vec<(String, String)> {
    let a: Vec<&str> = old.split_whitespace().collect();
    let b: Vec<&str> = new.split_whitespace().collect();
    let n = a.len();
    let m = b.len();

    // LCS length table (suffix form).
    let mut dp = vec![vec![0u16; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            dp[i][j] = if a[i] == b[j] {
                dp[i + 1][j + 1] + 1
            } else {
                dp[i + 1][j].max(dp[i][j + 1])
            };
        }
    }

    let mut pairs = Vec::new();
    let mut del: Vec<&str> = Vec::new();
    let mut ins: Vec<&str> = Vec::new();
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if a[i] == b[j] {
            flush(&mut del, &mut ins, &mut pairs);
            i += 1;
            j += 1;
        } else if dp[i + 1][j] >= dp[i][j + 1] {
            del.push(a[i]);
            i += 1;
        } else {
            ins.push(b[j]);
            j += 1;
        }
    }
    while i < n {
        del.push(a[i]);
        i += 1;
    }
    while j < m {
        ins.push(b[j]);
        j += 1;
    }
    flush(&mut del, &mut ins, &mut pairs);
    pairs
}

/// What one edit produced, for user feedback.
#[derive(Debug, Default, serde::Serialize)]
pub struct LearnOutcome {
    /// Substitutions observed in this edit.
    pub learned: Vec<[String; 2]>,
    /// Substitutions that crossed the threshold and became replacement rules now.
    pub promoted: Vec<[String; 2]>,
}

fn same(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// Record corrections from one edit into settings, promoting recurring ones to
/// replacement rules. Mutates `settings` (caller persists). Returns feedback.
pub fn record_corrections(settings: &mut AppSettings, pairs: &[(String, String)]) -> LearnOutcome {
    let mut outcome = LearnOutcome::default();
    for (from, to) in pairs {
        outcome.learned.push([from.clone(), to.clone()]);

        let count = {
            match settings
                .learned_corrections
                .iter_mut()
                .find(|c| same(&c.from, from) && c.to == *to)
            {
                Some(c) => {
                    c.count += 1;
                    c.count
                }
                None => {
                    settings.learned_corrections.push(LearnedCorrection {
                        from: from.clone(),
                        to: to.clone(),
                        count: 1,
                        promoted: false,
                    });
                    1
                }
            }
        };

        let already_promoted = settings
            .learned_corrections
            .iter()
            .any(|c| same(&c.from, from) && c.to == *to && c.promoted);

        if count >= PROMOTE_THRESHOLD && !already_promoted {
            let exists = settings
                .replacements
                .iter()
                .any(|r| !r.regex && same(&r.from, from) && r.to == *to);
            if !exists {
                settings.replacements.push(Replacement {
                    from: from.clone(),
                    to: to.clone(),
                    regex: false,
                });
            }
            if let Some(c) = settings
                .learned_corrections
                .iter_mut()
                .find(|c| same(&c.from, from) && c.to == *to)
            {
                c.promoted = true;
            }
            outcome.promoted.push([from.clone(), to.clone()]);
        }
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diff_multiword_to_single() {
        assert_eq!(
            word_diff("abre air table ya", "abre Airtable ya"),
            vec![("air table".to_string(), "Airtable".to_string())]
        );
    }

    #[test]
    fn diff_single_word_case_fix() {
        assert_eq!(
            word_diff("la cuenta de airtable", "la cuenta de Airtable"),
            vec![("airtable".to_string(), "Airtable".to_string())]
        );
    }

    #[test]
    fn diff_no_change_is_empty() {
        assert!(word_diff("hola mundo", "hola mundo").is_empty());
    }

    #[test]
    fn diff_pure_insertion_ignored() {
        assert!(word_diff("hola", "hola amigo").is_empty());
    }

    #[test]
    fn diff_pure_deletion_ignored() {
        assert!(word_diff("hola amigo", "hola").is_empty());
    }

    #[test]
    fn promotes_after_two_occurrences() {
        let mut s = AppSettings::default();
        let pairs = vec![("playrite".to_string(), "Playwright".to_string())];

        let o1 = record_corrections(&mut s, &pairs);
        assert!(o1.promoted.is_empty(), "first time should not promote");
        assert_eq!(s.replacements.len(), 0);
        assert_eq!(s.learned_corrections[0].count, 1);

        let o2 = record_corrections(&mut s, &pairs);
        assert_eq!(o2.promoted.len(), 1, "second time should promote");
        assert_eq!(s.replacements.len(), 1);
        assert_eq!(s.replacements[0].from, "playrite");
        assert_eq!(s.replacements[0].to, "Playwright");

        // Third time must not duplicate the rule.
        let _ = record_corrections(&mut s, &pairs);
        assert_eq!(s.replacements.len(), 1, "no duplicate rule");
    }
}
