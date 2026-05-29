//! Local project config (`.kotonoha/config.toml`) for M1 CLI.

use std::path::{Path, PathBuf};

const CONFIG_DIR: &str = ".kotonoha";
const CONFIG_FILE: &str = "config.toml";

#[derive(Debug, Clone)]
pub struct ProjectConfig {
    pub project_id: String,
}

#[derive(Debug)]
pub enum ProjectError {
    Io(std::io::Error),
    MissingProjectId,
    InvalidProjectId,
}

impl std::fmt::Display for ProjectError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProjectError::Io(e) => write!(f, "{e}"),
            ProjectError::MissingProjectId => {
                f.write_str("config missing project_id (run `kotonoha init`)")
            }
            ProjectError::InvalidProjectId => {
                f.write_str("config has project_id key but value is empty or malformed")
            }
        }
    }
}

impl std::error::Error for ProjectError {}

pub fn config_path(repo_root: &Path) -> PathBuf {
    repo_root.join(CONFIG_DIR).join(CONFIG_FILE)
}

pub fn init_config(
    repo_root: &Path,
    project_id: Option<&str>,
) -> Result<ProjectConfig, ProjectError> {
    let dir = repo_root.join(CONFIG_DIR);
    std::fs::create_dir_all(&dir).map_err(ProjectError::Io)?;
    let id = project_id
        .map(str::to_string)
        .unwrap_or_else(|| default_project_id(repo_root));
    let body = format!(
        "# Kotonoha local project config (non-normative)\nproject_id = \"{}\"\n",
        escape_toml_string(&id)
    );
    std::fs::write(config_path(repo_root), body).map_err(ProjectError::Io)?;
    Ok(ProjectConfig { project_id: id })
}

pub fn load_config(repo_root: &Path) -> Result<Option<ProjectConfig>, ProjectError> {
    let path = config_path(repo_root);
    if !path.is_file() {
        return Ok(None);
    }
    let text = std::fs::read_to_string(&path).map_err(ProjectError::Io)?;
    let id = match parse_project_id(&text) {
        Some(id) => id,
        None if text.lines().any(|l| {
            l.split('#')
                .next()
                .unwrap_or(l)
                .trim()
                .starts_with("project_id")
        }) =>
        {
            return Err(ProjectError::InvalidProjectId);
        }
        None => return Err(ProjectError::MissingProjectId),
    };
    Ok(Some(ProjectConfig { project_id: id }))
}

fn default_project_id(repo_root: &Path) -> String {
    repo_root
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("kotonoha-project")
        .to_string()
}

fn parse_project_id(text: &str) -> Option<String> {
    for line in text.lines() {
        let line = line.split('#').next().unwrap_or(line).trim();
        if let Some(rest) = line.strip_prefix("project_id") {
            let rest = rest.trim_start();
            if let Some(rest) = rest.strip_prefix('=') {
                let v = rest.trim().trim_matches('"').trim_matches('\'');
                if !v.is_empty() {
                    return Some(v.to_string());
                }
            }
        }
    }
    None
}

fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_project_id_from_toml() {
        let t = r#"
project_id = "my-repo"
"#;
        assert_eq!(parse_project_id(t).as_deref(), Some("my-repo"));
    }

    #[test]
    fn load_config_rejects_malformed_project_id() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        std::fs::create_dir_all(root.join(".kotonoha")).unwrap();
        std::fs::write(config_path(root), "project_id = \n").unwrap();
        let err = load_config(root).unwrap_err();
        assert!(matches!(err, ProjectError::InvalidProjectId));
    }
}
