use std::time::Duration;

use serde_json::Value;

const TIMEOUT: Duration = Duration::from_secs(5);

// ── Fallback hardcoded lists ────────────────────────────────────────

pub const FALLBACK_JDK: &[&str] = &["17", "21", "22", "23", "24", "25"];
pub const FALLBACK_NODE: &[&str] = &["18", "20", "22", "24"];
pub const FALLBACK_PYTHON: &[&str] = &["3.13", "3.12", "3.11", "3.10"];
pub const FALLBACK_GO: &[&str] = &["1.24", "1.23"];

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

pub fn parse_jdk_versions(body: &Value) -> Option<Vec<String>> {
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

pub fn parse_node_versions(body: &Value) -> Option<Vec<String>> {
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

pub fn parse_python_versions(body: &Value) -> Option<Vec<String>> {
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

pub fn parse_go_versions(body: &Value) -> Option<Vec<String>> {
    let releases = body.as_array()?;

    let mut minors: Vec<String> = releases
        .iter()
        .filter_map(|entry| {
            let version = entry.get("version")?.as_str()?;
            let stripped = version.strip_prefix("go")?;
            // Extract major.minor (e.g. "1.24" from "1.24.1")
            let parts: Vec<&str> = stripped.split('.').collect();
            if parts.len() >= 2
                && parts[0].chars().all(|c| c.is_ascii_digit())
                && parts[1].chars().all(|c| c.is_ascii_digit())
            {
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

pub fn fallback(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_string()).collect()
}

/// Simple check: is the given "YYYY-MM-DD" date in the past?
pub fn is_past_date(date_str: &str) -> bool {
    let now = current_date_iso();
    date_str <= now.as_str()
}

fn current_date_iso() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    date_from_epoch_secs(secs)
}

/// Convert epoch seconds to ISO date string (YYYY-MM-DD).
/// Uses Howard Hinnant's civil_from_days algorithm.
pub fn date_from_epoch_secs(secs: u64) -> String {
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
pub fn compare_version(a: &str, b: &str) -> std::cmp::Ordering {
    let parse = |s: &str| -> Vec<u64> {
        s.split('.').filter_map(|p| p.parse().ok()).collect()
    };
    parse(a).cmp(&parse(b))
}
