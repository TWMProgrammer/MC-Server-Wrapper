use super::types::*;
use ferinth::structures::{
    project::{Project, ProjectType},
    search::SearchHit,
    version::{Dependency, DependencyType, Version, VersionFile},
};

impl From<Project> for ModrinthProject {
    fn from(p: Project) -> Self {
        ModrinthProject {
            id: p.id,
            slug: p.slug,
            title: p.title,
            description: p.description,
            downloads: p.downloads as u64,
            icon_url: p.icon_url.map(|u| u.to_string()),
            screenshot_urls: Some(p.gallery.into_iter().map(|g| g.url.to_string()).collect()),
            author: "Unknown".to_string(),
            project_type: p.project_type.into(),
            categories: Some(p.categories),
            client_side: format!("{:?}", p.client_side).to_lowercase(),
            server_side: format!("{:?}", p.server_side).to_lowercase(),
        }
    }
}

impl From<SearchHit> for ModrinthProject {
    fn from(p: SearchHit) -> Self {
        ModrinthProject {
            id: p.project_id,
            slug: p.slug.unwrap_or_default(), // SearchHit slug is Option
            title: p.title,
            description: p.description,
            downloads: p.downloads as u64,
            icon_url: p.icon_url.map(|u: url::Url| u.to_string()),
            screenshot_urls: None,
            author: p.author,
            project_type: p.project_type.into(),
            categories: Some(p.categories),
            client_side: format!("{:?}", p.client_side).to_lowercase(),
            server_side: format!("{:?}", p.server_side).to_lowercase(),
        }
    }
}

impl From<Version> for ModrinthVersion {
    fn from(v: Version) -> Self {
        ModrinthVersion {
            id: v.id,
            project_id: v.project_id,
            version_number: v.version_number,
            files: v.files.into_iter().map(|f| f.into()).collect(),
            loaders: v.loaders,
            game_versions: v.game_versions,
            dependencies: v.dependencies.into_iter().map(|d| d.into()).collect(),
        }
    }
}

impl From<VersionFile> for ModrinthFile {
    fn from(f: VersionFile) -> Self {
        ModrinthFile {
            url: f.url.to_string(),
            filename: f.filename,
            primary: f.primary,
            size: f.size as u64,
            hashes: Some(ModrinthHashes {
                sha1: Some(f.hashes.sha1),
                sha512: Some(f.hashes.sha512),
            }),
        }
    }
}

impl From<Dependency> for ModrinthDependency {
    fn from(d: Dependency) -> Self {
        ModrinthDependency {
            project_id: d.project_id,
            version_id: d.version_id,
            dependency_type: match d.dependency_type {
                DependencyType::Required => "required".to_string(),
                DependencyType::Optional => "optional".to_string(),
                DependencyType::Incompatible => "incompatible".to_string(),
                DependencyType::Embedded => "embedded".to_string(),
            },
        }
    }
}

impl From<ProjectType> for ModrinthProjectType {
    fn from(pt: ProjectType) -> Self {
        match pt {
            ProjectType::Mod => ModrinthProjectType::Mod,
            ProjectType::Modpack => ModrinthProjectType::Modpack,
            ProjectType::ResourcePack => ModrinthProjectType::ResourcePack,
            ProjectType::Shader => ModrinthProjectType::Shader,
            ProjectType::Plugin => ModrinthProjectType::Plugin,
            ProjectType::Datapack => ModrinthProjectType::DataPack,
            ProjectType::Project => ModrinthProjectType::Mod, // Fallback
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ferinth::structures::project::ProjectType;

    #[test]
    fn test_project_type_conversion() {
        assert_eq!(
            ModrinthProjectType::from(ProjectType::Mod),
            ModrinthProjectType::Mod
        );
        assert_eq!(
            ModrinthProjectType::from(ProjectType::Modpack),
            ModrinthProjectType::Modpack
        );
        assert_eq!(
            ModrinthProjectType::from(ProjectType::ResourcePack),
            ModrinthProjectType::ResourcePack
        );
        assert_eq!(
            ModrinthProjectType::from(ProjectType::Shader),
            ModrinthProjectType::Shader
        );
        assert_eq!(
            ModrinthProjectType::from(ProjectType::Plugin),
            ModrinthProjectType::Plugin
        );
        assert_eq!(
            ModrinthProjectType::from(ProjectType::Datapack),
            ModrinthProjectType::DataPack
        );
        assert_eq!(
            ModrinthProjectType::from(ProjectType::Project),
            ModrinthProjectType::Mod
        );
    }
}
