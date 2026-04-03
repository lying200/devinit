use std::time::Duration;

use serde_json::Value;

const TIMEOUT: Duration = Duration::from_secs(5);

// ── Fallback hardcoded lists ────────────────────────────────────────

const FALLBACK_JDK: &[&str] = &["17", "21", "22", "23", "24", "25"];
const FALLBACK_NODE: &[&str] = &["18", "20", "22", "24"];
const FALLBACK_PYTHON: &[&str] = &["3.13", "3.12", "3.11", "3.10"];
const FALLBACK_GO: &[&str] = &["1.24", "1.23"];

// ── Public API ──────────────────────────────────────────────────────

/// Fetch available JDK major versions from Adoptium API.
/// Returns versions sorted ascending (e.g. `["17", "21", "22", ...]`).
#[must_use]
pub fn fetch_jdk_versions() -> Vec<String> {
    fetch_json("https://api.adoptium.net/v3/info/available_releases")
        .and_then(|body| parse_jdk_versions(&body))
        .unwrap_or_else(|| fallback(FALLBACK_JDK))
}

/// Fetch available Node.js LTS major versions.
/// Returns major versions sorted ascending (e.g. `["18", "20", "22", "24"]`).
#[must_use]
pub fn fetch_node_versions() -> Vec<String> {
    fetch_json("https://nodejs.org/dist/index.json")
        .and_then(|body| parse_node_versions(&body))
        .unwrap_or_else(|| fallback(FALLBACK_NODE))
}

/// Fetch active Python versions from endoflife.date.
/// Returns versions sorted ascending (e.g. `["3.10", "3.11", "3.12", "3.13"]`).
#[must_use]
pub fn fetch_python_versions() -> Vec<String> {
    fetch_json("https://endoflife.date/api/python.json")
        .and_then(|body| parse_python_versions(&body))
        .unwrap_or_else(|| fallback(FALLBACK_PYTHON))
}

/// Fetch currently maintained Go minor versions.
/// Returns versions sorted ascending (e.g. `["1.23", "1.24"]`).
#[must_use]
pub fn fetch_go_versions() -> Vec<String> {
    fetch_json("https://go.dev/dl/?mode=json")
        .and_then(|body| parse_go_versions(&body))
        .unwrap_or_else(|| fallback(FALLBACK_GO))
}

// ── Parsers (testable without network) ──────────────────────────────

fn parse_jdk_versions(body: &Value) -> Option<Vec<String>> {
    let releases = body.get("available_releases")?.as_array()?;
    let mut versions: Vec<String> = releases
        .iter()
        .filter_map(|v| v.as_u64().map(|n| n.to_string()))
        .collect();
    versions.sort_by_key(|v| v.parse::<u64>().unwrap_or(0));
    if versions.is_empty() {
        None
    } else {
        Some(versions)
    }
}

fn parse_node_versions(body: &Value) -> Option<Vec<String>> {
    let releases = body.as_array()?;

    let mut lts_majors: Vec<u64> = releases
        .iter()
        .filter(|entry| entry.get("lts").is_some_and(|v| v.is_string()))
        .filter_map(|entry| {
            let version = entry.get("version")?.as_str()?;
            let major = version.strip_prefix('v')?.split('.').next()?;
            major.parse::<u64>().ok()
        })
        .collect();

    lts_majors.sort();
    lts_majors.dedup();

    // Keep only recent LTS versions (last 4)
    if lts_majors.len() > 4 {
        lts_majors = lts_majors[lts_majors.len() - 4..].to_vec();
    }

    if lts_majors.is_empty() {
        None
    } else {
        Some(lts_majors.iter().map(|v| v.to_string()).collect())
    }
}

fn parse_python_versions(body: &Value) -> Option<Vec<String>> {
    let releases = body.as_array()?;

    let mut versions: Vec<String> = releases
        .iter()
        .filter(|entry| {
            // Only include versions that haven't reached EOL
            entry
                .get("eol")
                .and_then(|v| v.as_str())
                .is_some_and(|eol| !is_past_date(eol))
        })
        .filter_map(|entry| {
            let cycle = entry.get("cycle")?.as_str()?;
            // Only 3.x versions
            if cycle.starts_with("3.") {
                Some(cycle.to_string())
            } else {
                None
            }
        })
        .collect();

    versions.sort_by(|a, b| compare_version(a, b));

    if versions.is_empty() {
        None
    } else {
        Some(versions)
    }
}

fn parse_go_versions(body: &Value) -> Option<Vec<String>> {
    let releases = body.as_array()?;

    let mut minors: Vec<String> = releases
        .iter()
        .filter_map(|entry| {
            let version = entry.get("version")?.as_str()?;
            let stripped = version.strip_prefix("go")?;
            // Extract major.minor (e.g. "1.24" from "1.24.1")
            let parts: Vec<&str> = stripped.split('.').collect();
            if parts.len() >= 2 {
                Some(format!("{}.{}", parts[0], parts[1]))
            } else {
                None
            }
        })
        .collect();

    minors.sort_by(|a, b| compare_version(a, b));
    minors.dedup();

    if minors.is_empty() {
        None
    } else {
        Some(minors)
    }
}

// ── Helpers ─────────────────────────────────────────────────────────

fn fetch_json(url: &str) -> Option<Value> {
    let agent = ureq::Agent::config_builder()
        .timeout_global(Some(TIMEOUT))
        .build()
        .new_agent();
    let mut response = agent.get(url).call().ok()?;
    let body: Value = response.body_mut().read_json().ok()?;
    Some(body)
}

fn fallback(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// Simple check: is the given "YYYY-MM-DD" date in the past?
fn is_past_date(date_str: &str) -> bool {
    let now = current_date_iso();
    date_str <= now.as_str()
}

fn current_date_iso() -> String {
    // Use UNIX_EPOCH arithmetic to get current date without extra deps
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    date_from_epoch_secs(secs)
}

/// Convert epoch seconds to ISO date string (YYYY-MM-DD).
/// Uses Howard Hinnant's civil_from_days algorithm.
fn date_from_epoch_secs(secs: u64) -> String {
    let days = secs / 86400;
    let z = days + 719_468;
    let era = z / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{y:04}-{m:02}-{d:02}")
}

/// Compare dotted version strings numerically.
fn compare_version(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    parse(a).cmp(&parse(b))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── fallback ────────────────────────────────────────────────────

    #[test]
    fn fallback_returns_hardcoded_list() {
        let result = fallback(FALLBACK_JDK);
        assert_eq!(result, vec!["17", "21", "22", "23", "24", "25"]);
    }

    // ── compare_version ─────────────────────────────────────────────

    #[test]
    fn compare_version_orders_correctly() {
        assert_eq!(compare_version("3.10", "3.9"), std::cmp::Ordering::Greater);
        assert_eq!(compare_version("1.23", "1.24"), std::cmp::Ordering::Less);
        assert_eq!(compare_version("3.12", "3.12"), std::cmp::Ordering::Equal);
    }

    #[test]
    fn compare_version_different_lengths() {
        assert_eq!(compare_version("1.2", "1.2.3"), std::cmp::Ordering::Less);
        assert_eq!(compare_version("1.2.3", "1.2"), std::cmp::Ordering::Greater);
    }

    #[test]
    fn compare_version_single_segment() {
        assert_eq!(compare_version("9", "11"), std::cmp::Ordering::Less);
    }

    // ── date_from_epoch_secs ────────────────────────────────────────

    #[test]
    fn date_from_epoch_secs_unix_epoch() {
        assert_eq!(date_from_epoch_secs(0), "1970-01-01");
    }

    #[test]
    fn date_from_epoch_secs_known_date() {
        // 2024-01-01 00:00:00 UTC = 1704067200
        assert_eq!(date_from_epoch_secs(1_704_067_200), "2024-01-01");
    }

    #[test]
    fn date_from_epoch_secs_leap_day() {
        // 2024-02-29 00:00:00 UTC = 1709164800
        assert_eq!(date_from_epoch_secs(1_709_164_800), "2024-02-29");
    }

    #[test]
    fn date_from_epoch_secs_year_boundary() {
        // 2023-12-31 23:59:59 UTC = 1704067199
        assert_eq!(date_from_epoch_secs(1_704_067_199), "2023-12-31");
    }

    #[test]
    fn date_from_epoch_secs_2026() {
        // 2026-04-03 00:00:00 UTC = 1775174400
        assert_eq!(date_from_epoch_secs(1_775_174_400), "2026-04-03");
    }

    // ── is_past_date ────────────────────────────────────────────────

    #[test]
    fn is_past_date_works() {
        assert!(is_past_date("2020-01-01"));
        assert!(!is_past_date("2099-01-01"));
    }

    // ── parse_jdk_versions ──────────────────────────────────────────

    #[test]
    fn parse_jdk_versions_normal() {
        let body = json!({
            "available_releases": [8, 11, 17, 21, 22, 23, 24, 25],
            "most_recent_lts": 25
        });
        let result = parse_jdk_versions(&body).unwrap();
        assert_eq!(result, vec!["8", "11", "17", "21", "22", "23", "24", "25"]);
    }

    #[test]
    fn parse_jdk_versions_empty_array() {
        let body = json!({"available_releases": []});
        assert_eq!(parse_jdk_versions(&body), None);
    }

    #[test]
    fn parse_jdk_versions_missing_field() {
        let body = json!({"other_field": [1, 2]});
        assert_eq!(parse_jdk_versions(&body), None);
    }

    #[test]
    fn parse_jdk_versions_non_numeric_filtered() {
        let body = json!({"available_releases": [17, "ea", 21, null]});
        let result = parse_jdk_versions(&body).unwrap();
        assert_eq!(result, vec!["17", "21"]);
    }

    #[test]
    fn parse_jdk_versions_sorted() {
        let body = json!({"available_releases": [21, 8, 17, 11]});
        let result = parse_jdk_versions(&body).unwrap();
        assert_eq!(result, vec!["8", "11", "17", "21"]);
    }

    // ── parse_node_versions ─────────────────────────────────────────

    #[test]
    fn parse_node_versions_filters_lts_only() {
        let body = json!([
            {"version": "v25.0.0", "lts": false},
            {"version": "v24.11.0", "lts": "Krypton"},
            {"version": "v22.5.0", "lts": "Jod"},
            {"version": "v21.0.0", "lts": false},
            {"version": "v20.10.0", "lts": "Iron"},
            {"version": "v18.19.0", "lts": "Hydrogen"}
        ]);
        let result = parse_node_versions(&body).unwrap();
        assert_eq!(result, vec!["18", "20", "22", "24"]);
    }

    #[test]
    fn parse_node_versions_dedupes_majors() {
        let body = json!([
            {"version": "v22.5.0", "lts": "Jod"},
            {"version": "v22.4.0", "lts": "Jod"},
            {"version": "v20.10.0", "lts": "Iron"}
        ]);
        let result = parse_node_versions(&body).unwrap();
        assert_eq!(result, vec!["20", "22"]);
    }

    #[test]
    fn parse_node_versions_limits_to_four() {
        let body = json!([
            {"version": "v24.11.0", "lts": "Krypton"},
            {"version": "v22.5.0", "lts": "Jod"},
            {"version": "v20.10.0", "lts": "Iron"},
            {"version": "v18.19.0", "lts": "Hydrogen"},
            {"version": "v16.20.0", "lts": "Gallium"}
        ]);
        let result = parse_node_versions(&body).unwrap();
        assert_eq!(result, vec!["18", "20", "22", "24"]);
    }

    #[test]
    fn parse_node_versions_empty_returns_none() {
        let body = json!([
            {"version": "v25.0.0", "lts": false}
        ]);
        assert_eq!(parse_node_versions(&body), None);
    }

    #[test]
    fn parse_node_versions_not_array_returns_none() {
        let body = json!({"versions": []});
        assert_eq!(parse_node_versions(&body), None);
    }

    // ── parse_python_versions ───────────────────────────────────────

    #[test]
    fn parse_python_versions_filters_eol() {
        let body = json!([
            {"cycle": "3.13", "eol": "2029-10-01"},
            {"cycle": "3.12", "eol": "2028-10-01"},
            {"cycle": "3.8", "eol": "2024-10-01"},
            {"cycle": "2.7", "eol": "2020-01-01"}
        ]);
        let result = parse_python_versions(&body).unwrap();
        assert_eq!(result, vec!["3.12", "3.13"]);
    }

    #[test]
    fn parse_python_versions_excludes_python2() {
        let body = json!([
            {"cycle": "3.13", "eol": "2029-10-01"},
            {"cycle": "2.8", "eol": "2099-01-01"}
        ]);
        let result = parse_python_versions(&body).unwrap();
        assert_eq!(result, vec!["3.13"]);
    }

    #[test]
    fn parse_python_versions_sorted_numerically() {
        let body = json!([
            {"cycle": "3.9", "eol": "2099-01-01"},
            {"cycle": "3.12", "eol": "2099-01-01"},
            {"cycle": "3.10", "eol": "2099-01-01"}
        ]);
        let result = parse_python_versions(&body).unwrap();
        assert_eq!(result, vec!["3.9", "3.10", "3.12"]);
    }

    #[test]
    fn parse_python_versions_all_eol_returns_none() {
        let body = json!([
            {"cycle": "3.7", "eol": "2023-06-27"},
            {"cycle": "2.7", "eol": "2020-01-01"}
        ]);
        assert_eq!(parse_python_versions(&body), None);
    }

    // ── parse_go_versions ───────────────────────────────────────────

    #[test]
    fn parse_go_versions_extracts_minors() {
        let body = json!([
            {"version": "go1.24.1"},
            {"version": "go1.23.5"}
        ]);
        let result = parse_go_versions(&body).unwrap();
        assert_eq!(result, vec!["1.23", "1.24"]);
    }

    #[test]
    fn parse_go_versions_dedupes() {
        let body = json!([
            {"version": "go1.24.1"},
            {"version": "go1.24.0"},
            {"version": "go1.23.5"}
        ]);
        let result = parse_go_versions(&body).unwrap();
        assert_eq!(result, vec!["1.23", "1.24"]);
    }

    #[test]
    fn parse_go_versions_skips_no_go_prefix() {
        let body = json!([
            {"version": "1.24.1"},
            {"version": "go1.23.5"}
        ]);
        let result = parse_go_versions(&body).unwrap();
        assert_eq!(result, vec!["1.23"]);
    }

    #[test]
    fn parse_go_versions_skips_single_part() {
        let body = json!([
            {"version": "go1"},
            {"version": "go1.23.5"}
        ]);
        let result = parse_go_versions(&body).unwrap();
        assert_eq!(result, vec!["1.23"]);
    }

    #[test]
    fn parse_go_versions_empty_returns_none() {
        let body = json!([]);
        assert_eq!(parse_go_versions(&body), None);
    }
}
