use crate::types::project::Project;
use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

pub struct JsonProject;

impl JsonProject {
    pub fn to_string(project: &Project) -> Result<String> {
        let json = serde_json::to_string_pretty(project)?;
        Ok(json)
    }

    pub fn save(project: &Project, path: &Path) -> Result<()> {
        let json = Self::to_string(project)?;
        fs::write(path, json)
            .with_context(|| format!("Failed to save project to {}", path.display()))?;
        Ok(())
    }

    pub fn from_str(json: &str) -> Result<Project> {
        let project: Project = serde_json::from_str(json)?;
        Ok(project)
    }

    pub fn load(path: &Path) -> Result<Project> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to load project from {}", path.display()))?;
        Self::from_str(&content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_json_roundtrip() {
        let project = Project::new("proj-123", "Demo Video");
        let json = JsonProject::to_string(&project).unwrap();
        let loaded = JsonProject::from_str(&json).unwrap();

        assert_eq!(loaded.id, "proj-123");
        assert_eq!(loaded.name, "Demo Video");
        assert_eq!(loaded.schema_version, Project::CURRENT_SCHEMA_VERSION);
    }
}
