use devinit::detection::detect_services;
use devinit::schema::Service;
use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

fn unique_test_dir(name: &str) -> PathBuf {
    let pid = std::process::id();
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "devinit-services-detect-{name}-{pid}-{nanos}"
    ))
}

fn create_dir(path: &Path) {
    fs::create_dir_all(path).unwrap();
}

#[test]
fn detect_services_from_docker_compose_yml() {
    let dir = unique_test_dir("compose-yml");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    image: postgres:16\n  cache:\n    image: redis:7\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 2);
    assert!(candidates
        .iter()
        .any(|c| matches!(c.service, Service::Postgres { .. })));
    assert!(candidates
        .iter()
        .any(|c| matches!(c.service, Service::Redis)));
}

#[test]
fn detect_services_from_compose_yml() {
    let dir = unique_test_dir("compose-short");
    create_dir(&dir);
    fs::write(
        dir.join("compose.yml"),
        "services:\n  db:\n    image: mysql:8\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Mysql { .. }));
}

#[test]
fn detect_services_returns_empty_without_compose() {
    let dir = unique_test_dir("no-compose");
    create_dir(&dir);

    let candidates = detect_services(&dir).unwrap();
    assert!(candidates.is_empty());
}

#[test]
fn detect_services_ignores_unknown_images() {
    let dir = unique_test_dir("unknown-images");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  web:\n    image: nginx:latest\n  app:\n    image: node:20\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert!(candidates.is_empty());
}

#[test]
fn detect_services_from_subdirectory() {
    let dir = unique_test_dir("subdir-compose");
    create_dir(&dir);
    create_dir(&dir.join("infra"));
    fs::write(
        dir.join("infra/docker-compose.yml"),
        "services:\n  db:\n    image: postgres:16\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Postgres { .. }));
}

#[test]
fn detect_services_deduplicates_same_service() {
    let dir = unique_test_dir("dedup-services");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db1:\n    image: postgres:15\n  db2:\n    image: postgres:16\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    let pg_count = candidates
        .iter()
        .filter(|c| matches!(c.service, Service::Postgres { .. }))
        .count();
    assert_eq!(pg_count, 1);
}

#[test]
fn detect_services_handles_mariadb_as_mysql() {
    let dir = unique_test_dir("mariadb");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    image: mariadb:11\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Mysql { .. }));
}

#[test]
fn detect_services_handles_quoted_images() {
    let dir = unique_test_dir("quoted-images");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    image: \"postgres:16\"\n  cache:\n    image: 'redis:7'\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 2);
}

#[test]
fn detect_services_handles_registry_prefix() {
    let dir = unique_test_dir("registry-prefix");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    image: docker.io/library/postgres:16\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Postgres { .. }));
}

#[test]
fn detect_services_from_docker_compose_yaml() {
    let dir = unique_test_dir("compose-yaml-ext");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yaml"),
        "services:\n  db:\n    image: postgres:16\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Postgres { .. }));
}

#[test]
fn detect_services_from_compose_yaml() {
    let dir = unique_test_dir("compose-yaml-short");
    create_dir(&dir);
    fs::write(
        dir.join("compose.yaml"),
        "services:\n  cache:\n    image: redis:7\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Redis));
}

#[test]
fn detect_services_prefers_first_compose_file() {
    let dir = unique_test_dir("multi-compose");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    image: postgres:16\n",
    )
    .unwrap();
    fs::write(
        dir.join("compose.yml"),
        "services:\n  cache:\n    image: redis:7\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    // Only docker-compose.yml should be picked (first match wins per dir)
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Postgres { .. }));
}

#[test]
fn detect_services_empty_compose_file() {
    let dir = unique_test_dir("empty-compose");
    create_dir(&dir);
    fs::write(dir.join("docker-compose.yml"), "").unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert!(candidates.is_empty());
}

#[test]
fn detect_services_all_three_services() {
    let dir = unique_test_dir("all-three");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    image: postgres:16\n  cache:\n    image: redis:7\n  mysql:\n    image: mysql:8\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 3);
    assert!(candidates.iter().any(|c| matches!(c.service, Service::Postgres { .. })));
    assert!(candidates.iter().any(|c| matches!(c.service, Service::Redis)));
    assert!(candidates.iter().any(|c| matches!(c.service, Service::Mysql { .. })));
}

#[test]
fn detect_services_ignores_commented_lines() {
    let dir = unique_test_dir("commented");
    create_dir(&dir);
    fs::write(
        dir.join("docker-compose.yml"),
        "services:\n  db:\n    # image: postgres:16\n    image: redis:7\n",
    )
    .unwrap();

    let candidates = detect_services(&dir).unwrap();
    assert_eq!(candidates.len(), 1);
    assert!(matches!(candidates[0].service, Service::Redis));
}
