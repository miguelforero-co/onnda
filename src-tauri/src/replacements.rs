//! Deterministic post-transcription find/replace rules and voice snippets.
//!
//! Applied to every transcription (dictation + file) for BOTH engines, after
//! vocabulary correction. This is the primary way custom vocabulary reaches the
//! Apple SpeechAnalyzer engine, which (unlike Whisper) has no initial_prompt:
//! e.g. it tends to write "Air Table" / "Playrite" — a rule fixes those to
//! "Airtable" / "Playwright". Snippets are just replacements too:
//! "mi correo" -> "hello@example.com".

use regex::Regex;

use crate::settings::Replacement;

/// Apply all replacement rules in order. Literal rules are case-insensitive and
/// may span multiple words; regex rules use `from` as the pattern verbatim.
/// Invalid regex rules are skipped (never panic on user input).
pub fn apply_replacements(text: &str, rules: &[Replacement]) -> String {
    let mut out = text.to_string();
    for r in rules {
        if r.from.is_empty() {
            continue;
        }
        let pattern = if r.regex {
            r.from.clone()
        } else {
            // Case-insensitive literal match (multi-word allowed).
            format!("(?i){}", regex::escape(&r.from))
        };
        if let Ok(re) = Regex::new(&pattern) {
            out = if r.regex {
                // Regex rules may reference capture groups ($1, ${name}).
                re.replace_all(&out, r.to.as_str()).into_owned()
            } else {
                // Literal rules / snippets must be inserted verbatim — no $-expansion
                // (so "cuesta $5" or an email isn't mangled).
                re.replace_all(&out, regex::NoExpand(r.to.as_str())).into_owned()
            };
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lit(from: &str, to: &str) -> Replacement {
        Replacement { from: from.into(), to: to.into(), regex: false }
    }
    fn rx(from: &str, to: &str) -> Replacement {
        Replacement { from: from.into(), to: to.into(), regex: true }
    }

    #[test]
    fn literal_is_case_insensitive_and_multiword() {
        let rules = vec![lit("air table", "Airtable"), lit("java script", "JavaScript")];
        let out = apply_replacements("Abre Air Table y usa Java Script", &rules);
        assert_eq!(out, "Abre Airtable y usa JavaScript");
    }

    #[test]
    fn snippet_expansion() {
        let rules = vec![lit("mi correo", "hello@example.com")];
        assert_eq!(
            apply_replacements("escríbele a mi correo por favor", &rules),
            "escríbele a hello@example.com por favor"
        );
    }

    #[test]
    fn regex_rule_applies() {
        // Collapse repeated filler "eh eh" -> "" and normalize.
        let rules = vec![rx(r"\b(eh|este)\b\s*", "")];
        assert_eq!(apply_replacements("bueno eh este vamos", &rules), "bueno vamos");
    }

    #[test]
    fn invalid_regex_is_skipped_not_panicking() {
        let rules = vec![rx("(", "x"), lit("hola", "chau")];
        assert_eq!(apply_replacements("hola", &rules), "chau");
    }

    #[test]
    fn literal_to_with_dollar_is_verbatim() {
        // "$5" must not be treated as a capture-group reference.
        let rules = vec![lit("mi precio", "cuesta $5 cada uno")];
        assert_eq!(
            apply_replacements("dile mi precio", &rules),
            "dile cuesta $5 cada uno"
        );
    }

    #[test]
    fn empty_from_is_ignored() {
        let rules = vec![lit("", "x")];
        assert_eq!(apply_replacements("sin cambios", &rules), "sin cambios");
    }

    #[test]
    fn rules_apply_in_order() {
        let rules = vec![lit("a", "b"), lit("b", "c")];
        // "a" -> "b" -> "c"
        assert_eq!(apply_replacements("a", &rules), "c");
    }
}
