//! Built-in narrative strategies: ordered sequences of shared slide roles.
//! Data lives in `data/roles.json` and `data/narratives/*.json`.

use rust_embed::Embed;
use serde_json::Value;

#[derive(Embed)]
#[folder = "data/"]
struct Data;

const FAMILIES: &[&str] = &["logic", "story", "persuasion", "report", "pitch"];
pub const AUDIENCES: &[&str] = &["executive", "team", "technical", "customer", "investor", "general", "academic"];
pub const GOALS: &[&str] = &["decide", "persuade", "inform", "inspire", "sell", "report", "teach", "fund"];

fn json(path: &str) -> Result<Value, String> {
    let file = Data::get(path).ok_or_else(|| format!("{path}: missing"))?;
    serde_json::from_slice(&file.data).map_err(|e| format!("{path}: {e}"))
}

pub fn roles() -> Result<Value, String> {
    json("roles.json")
}

/// All strategies, sorted by id, each validated against the role registry.
pub fn strategies() -> Result<Vec<Value>, String> {
    let roles = roles()?;
    let mut paths: Vec<String> = Data::iter()
        .filter(|p| p.starts_with("narratives/") && p.ends_with(".json"))
        .map(|p| p.to_string())
        .collect();
    paths.sort();
    paths.iter().map(|p| json(p).and_then(|v| validate(&v, p, &roles).map(|_| v))).collect()
}

pub fn find(id: &str) -> Result<Value, String> {
    strategies()?
        .into_iter()
        .find(|s| s["id"] == id)
        .ok_or_else(|| format!("unknown strategy {id}; see `wb-slide narrative list`"))
}

fn text(v: &Value, key: &str, path: &str) -> Result<(), String> {
    match v[key].as_str() {
        Some(s) if !s.trim().is_empty() => Ok(()),
        _ => Err(format!("{path}: {key} must be a non-empty string")),
    }
}

fn vocab(v: &Value, allowed: &[&str], path: &str) -> Result<(), String> {
    let items = v.as_array().filter(|a| !a.is_empty()).ok_or(format!("{path}: expected a non-empty array"))?;
    for i in items {
        if !i.as_str().is_some_and(|s| allowed.contains(&s)) {
            return Err(format!("{path}: {i} not in {allowed:?}"));
        }
    }
    Ok(())
}

fn validate(s: &Value, path: &str, roles: &Value) -> Result<(), String> {
    let keys = ["id", "name", "family", "summary", "references", "when_to_use", "title_style", "density", "sequence", "pitfalls", "example"];
    let obj = s.as_object().ok_or(format!("{path}: expected an object"))?;
    if let Some(k) = obj.keys().find(|k| !keys.contains(&k.as_str())) {
        return Err(format!("{path}: unknown key {k}"));
    }
    for k in ["id", "name", "summary", "title_style"] {
        text(s, k, path)?;
    }
    if format!("narratives/{}.json", s["id"].as_str().unwrap()) != path {
        return Err(format!("{path}: id must match the file name"));
    }
    vocab(&Value::from(vec![s["family"].clone()]), FAMILIES, &format!("{path}: family"))?;
    let w = &s["when_to_use"];
    vocab(&w["audiences"], AUDIENCES, &format!("{path}: audiences"))?;
    vocab(&w["goals"], GOALS, &format!("{path}: goals"))?;
    match (w["minutes"][0].as_u64(), w["minutes"][1].as_u64()) {
        (Some(a), Some(b)) if a <= b => {}
        _ => return Err(format!("{path}: minutes must be [min, max]")),
    }
    let seq = s["sequence"].as_array().filter(|a| (3..=12).contains(&a.len())).ok_or(format!("{path}: sequence needs 3-12 entries"))?;
    for (i, step) in seq.iter().enumerate() {
        let role = step["role"].as_str().unwrap_or_default();
        if roles["roles"].get(role).is_none() {
            return Err(format!("{path}: sequence[{i}] unknown role {role:?}"));
        }
        text(step, "purpose", &format!("{path}: sequence[{i}]"))?;
        if let Some(l) = step.get("layout") {
            if roles["layouts"].get(l.as_str().unwrap_or_default()).is_none() {
                return Err(format!("{path}: sequence[{i}] unknown layout {l}"));
            }
        }
    }
    let outline = s["example"]["outline"].as_array().ok_or(format!("{path}: example.outline missing"))?;
    if outline.len() != seq.len() {
        return Err(format!("{path}: example.outline must have one line per sequence entry"));
    }
    text(&s["example"], "topic", path)?;
    for k in ["references", "pitfalls"] {
        if !s[k].as_array().is_some_and(|a| !a.is_empty() && a.iter().all(|x| x.as_str().is_some_and(|t| !t.trim().is_empty()))) {
            return Err(format!("{path}: {k} must be a non-empty array of strings"));
        }
    }
    let d = &s["density"];
    if d["max_bullets"].as_u64().is_none() || d["max_words"].as_u64().is_none() || d["one_idea_per_slide"].as_bool().is_none() {
        return Err(format!("{path}: density needs max_bullets, max_words, one_idea_per_slide"));
    }
    Ok(())
}

pub fn list() -> Result<String, String> {
    let mut out = String::new();
    for s in strategies()? {
        out += &format!("{:<28} {:<11} {}\n", s["id"].as_str().unwrap(), s["family"].as_str().unwrap(), s["name"].as_str().unwrap());
    }
    Ok(out)
}

pub fn show(id: &str) -> Result<String, String> {
    serde_json::to_string_pretty(&find(id)?).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_strategy_is_valid_and_uses_shared_roles() {
        let all = strategies().unwrap();
        assert!(!all.is_empty());
        assert!(list().unwrap().contains("scqa"));
        assert!(show("scqa").unwrap().contains("\"complication\""));
        assert!(show("nope").unwrap_err().contains("unknown strategy"));
    }

    #[test]
    fn validator_rejects_unknown_role() {
        let roles = roles().unwrap();
        let mut s = find("scqa").unwrap();
        s["sequence"][0]["role"] = "made-up".into();
        assert!(validate(&s, "narratives/scqa.json", &roles).unwrap_err().contains("unknown role"));
    }
}
