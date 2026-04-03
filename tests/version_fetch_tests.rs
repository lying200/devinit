use devinit::version_fetch::{
    FALLBACK_JDK, compare_version, date_from_epoch_secs, fallback, is_past_date,
    parse_go_versions, parse_jdk_versions, parse_node_versions, parse_python_versions,
};
use serde_json::json;

// ── fallback ────────────────────────────────────────────────────────

#[test]
fn fallback_returns_hardcoded_list() {
    let result = fallback(FALLBACK_JDK);
    let expected: Vec<String> = FALLBACK_JDK.iter().map(|s| s.to_string()).collect();
    assert_eq!(result, expected);
}

// ── compare_version ─────────────────────────────────────────────────

#[test]
fn compare_version_orders_correctly() {
    assert_eq!(compare_version("3.10", "3.9"), std::cmp::Ordering::Greater);
    assert_eq!(compare_version("1.23", "1.24"), std::cmp::Ordering::Less);
    assert_eq!(compare_version("3.12", "3.12"), std::cmp::Ordering::Equal);
}

#[test]
fn compare_version_different_lengths() {
    assert_eq!(compare_version("1.2", "1.2.3"), std::cmp::Ordering::Less);
    assert_eq!(
        compare_version("1.2.3", "1.2"),
        std::cmp::Ordering::Greater
    );
}

#[test]
fn compare_version_single_segment() {
    assert_eq!(compare_version("9", "11"), std::cmp::Ordering::Less);
}

// ── date_from_epoch_secs ────────────────────────────────────────────

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

// ── is_past_date ────────────────────────────────────────────────────

#[test]
fn is_past_date_works() {
    assert!(is_past_date("2020-01-01"));
    assert!(!is_past_date("2099-01-01"));
}

// ── parse_jdk_versions ──────────────────────────────────────────────

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

// ── parse_node_versions ─────────────────────────────────────────────

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

// ── parse_python_versions ───────────────────────────────────────────

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

// ── parse_go_versions ───────────────────────────────────────────────

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
fn parse_go_versions_skips_rc_versions() {
    let body = json!([
        {"version": "go1.25rc1"},
        {"version": "go1.24.1"}
    ]);
    let result = parse_go_versions(&body).unwrap();
    assert_eq!(result, vec!["1.24"]);
}

#[test]
fn parse_go_versions_empty_returns_none() {
    let body = json!([]);
    assert_eq!(parse_go_versions(&body), None);
}
