pub use crate::audio::rms_f32;

pub fn resample(samples: &[f32], from_hz: usize, to_hz: usize) -> Vec<f32> {
    if from_hz == to_hz || samples.is_empty() { return samples.to_vec(); }
    let ratio = from_hz as f64 / to_hz as f64;
    let out_len = (samples.len() as f64 / ratio) as usize;
    (0..out_len).map(|i| {
        let pos = i as f64 * ratio;
        let lo = pos as usize;
        let hi = (lo + 1).min(samples.len() - 1);
        let frac = (pos - lo as f64) as f32;
        samples[lo] * (1.0 - frac) + samples[hi] * frac
    }).collect()
}

/// Replace transcribed words with custom vocabulary entries when they are close matches.
/// Scans the text word-by-word; replaces a run of 1–3 consecutive words if their
/// Jaro-Winkler similarity to a vocab entry meets the threshold.
/// Handles multi-word entries like "Claude Code" or "Node.js".
pub fn correct_words(text: &str, vocab_csv: &str, threshold: f32) -> String {
    if vocab_csv.is_empty() || threshold >= 1.0 { return text.to_string(); }

    let vocab: Vec<String> = vocab_csv
        .split([',', '\n'])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    if vocab.is_empty() { return text.to_string(); }

    let tokens: Vec<(String, String, String)> = text
        .split_whitespace()
        .map(|w| {
            let lead: String = w.chars().take_while(|c| !c.is_alphanumeric()).collect();
            // Compute the trailing punctuation over what's LEFT after the lead,
            // so a token that is all punctuation (e.g. "—" or "...") doesn't count
            // the same chars as both lead and trail (which underflowed the slice).
            let after_lead = &w[lead.len()..];
            let trail: String = after_lead
                .chars()
                .rev()
                .take_while(|c| !c.is_alphanumeric())
                .collect::<String>()
                .chars()
                .rev()
                .collect();
            let core = w[lead.len()..w.len() - trail.len()].to_string();
            (lead, core, trail)
        })
        .collect();

    let n = tokens.len();
    let mut out: Vec<String> = Vec::with_capacity(n);
    let mut i = 0;

    while i < n {
        let mut matched = false;
        'outer: for window in [3usize, 2, 1] {
            if i + window > n { continue; }
            let candidate: String = tokens[i..i + window]
                .iter()
                .map(|(_, core, _)| core.as_str())
                .collect::<Vec<_>>()
                .join(" ");

            for entry in &vocab {
                if jaro_winkler_match(&candidate, entry, threshold) {
                    let lead = &tokens[i].0;
                    let trail = &tokens[i + window - 1].2;
                    out.push(format!("{}{}{}", lead, entry, trail));
                    i += window;
                    matched = true;
                    break 'outer;
                }
            }
        }
        if !matched {
            let (lead, core, trail) = &tokens[i];
            out.push(format!("{}{}{}", lead, core, trail));
            i += 1;
        }
    }

    out.join(" ")
}

fn jaro_winkler_match(candidate: &str, entry: &str, threshold: f32) -> bool {
    let a = candidate.to_lowercase();
    let b = entry.to_lowercase();
    if a == b { return true; }
    if a.len() <= 3 || b.len() <= 3 { return false; }
    strsim::jaro_winkler(&a, &b) >= threshold as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    // Regression: a token that is all punctuation used to panic the slice in
    // correct_words (lead and trail counted the same chars).
    #[test]
    fn punctuation_only_tokens_do_not_panic() {
        for t in ["—", "...", "¿?", "!!!", "hola — mundo", "fin..."] {
            let _ = correct_words(t, "Claude", 0.85);
        }
    }

    #[test]
    fn replaces_close_match_preserving_punctuation() {
        let out = correct_words("usé cloud code hoy.", "Claude Code", 0.82);
        assert!(out.contains("Claude Code"), "got: {out}");
        assert!(out.ends_with("hoy."), "punctuation lost: {out}");
    }

    #[test]
    fn empty_vocab_is_identity() {
        assert_eq!(correct_words("texto cualquiera", "", 0.85), "texto cualquiera");
    }
}
