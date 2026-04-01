use std::path::{Path, PathBuf};
use std::{fs, io};

use crate::schema::Service;

use super::detectors::scannable_dirs;

const COMPOSE_FILES: &[&str] = &[
    "docker-compose.yml",
    "docker-compose.yaml",
    "compose.yml",
    "compose.yaml",
];

#[derive(Debug, Clone)]
pub struct ServiceCandidate {
    pub service: Service,
    pub reasons: Vec<String>,
}

/// Detects services from docker-compose files in `target_dir` and its
/// immediate subdirectories.
///
/// # Errors
///
/// Returns any I/O error from reading directories or compose files.
pub fn detect_services(target_dir: &Path) -> io::Result<Vec<ServiceCandidate>> {
    let dirs = scannable_dirs(target_dir)?;
    let mut candidates = Vec::new();

    for dir in &dirs {
        if let Some(path) = find_compose_file(dir) {
            let content = fs::read_to_string(&path)?;
            let display = path.display().to_string();
            parse_compose_services(&content, &display, &mut candidates);
        }
    }

    dedup_candidates(&mut candidates);
    Ok(candidates)
}

fn find_compose_file(dir: &Path) -> Option<PathBuf> {
    for name in COMPOSE_FILES {
        let path = dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }
    None
}

fn parse_compose_services(content: &str, source: &str, candidates: &mut Vec<ServiceCandidate>) {
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('#') {
            continue;
        }
        let Some(image) = trimmed
            .strip_prefix("image:")
            .or_else(|| trimmed.strip_prefix("image :"))
        else {
            continue;
        };

        let image = image.trim().trim_matches('"').trim_matches('\'');
        // Strip registry prefix: library/postgres → postgres, docker.io/library/redis → redis
        let image_name = image
            .rsplit('/')
            .next()
            .unwrap_or(image)
            .split(':')
            .next()
            .unwrap_or(image);

        match image_name {
            "postgres" | "postgresql" => {
                candidates.push(ServiceCandidate {
                    service: Service::Postgres { package: None },
                    reasons: vec![format!("found image {image} in {source}")],
                });
            }
            "redis" => {
                candidates.push(ServiceCandidate {
                    service: Service::Redis,
                    reasons: vec![format!("found image {image} in {source}")],
                });
            }
            "mysql" | "mariadb" => {
                candidates.push(ServiceCandidate {
                    service: Service::Mysql { package: None },
                    reasons: vec![format!("found image {image} in {source}")],
                });
            }
            _ => {}
        }
    }
}

fn dedup_candidates(candidates: &mut Vec<ServiceCandidate>) {
    let mut seen = Vec::new();
    candidates.retain(|c| {
        let key = std::mem::discriminant(&c.service);
        if seen.contains(&key) {
            false
        } else {
            seen.push(key);
            true
        }
    });
}
