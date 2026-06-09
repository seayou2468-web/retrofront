use crate::core_info::{CoreInfo, CoreInfoList};
use crate::settings::Settings;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchDecisionKind {
    NoCore,
    Selected,
    NeedsCoreChoice,
}

#[derive(Debug, Clone)]
pub struct LaunchPlan {
    pub content_path: PathBuf,
    pub content_extension: String,
    pub candidates: Vec<CoreInfo>,
    pub selected_core: Option<PathBuf>,
    pub decision: LaunchDecisionKind,
    pub reason: String,
}

impl LaunchPlan {
    pub fn selected(content_path: PathBuf, content_extension: String, core: CoreInfo) -> Self {
        let selected_core = Some(core.path.clone());
        Self {
            content_path,
            content_extension,
            candidates: vec![core],
            selected_core,
            decision: LaunchDecisionKind::Selected,
            reason: "one compatible core found".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct LaunchManager {
    pub last_plan: Option<LaunchPlan>,
}

impl LaunchManager {
    pub fn new() -> Self {
        Self { last_plan: None }
    }

    pub fn plan_content_launch(
        &mut self,
        content_path: &Path,
        core_info: &CoreInfoList,
        settings: &Settings,
        requested_core: Option<&Path>,
    ) -> LaunchPlan {
        let content_extension = extension_for_content(content_path);
        let candidates = if let Some(core_path) = requested_core {
            core_info
                .cores
                .iter()
                .find(|core| paths_equal(&core.path, core_path))
                .cloned()
                .into_iter()
                .collect()
        } else {
            core_info.compatible_cores_for_content_path(content_path)
        };

        let preferred = settings.preferred_core_for_extension(&content_extension);
        let plan = choose_core(
            content_path,
            content_extension,
            candidates,
            preferred.as_deref(),
        );
        self.last_plan = Some(plan.clone());
        plan
    }
}

fn choose_core(
    content_path: &Path,
    content_extension: String,
    candidates: Vec<CoreInfo>,
    preferred_core: Option<&Path>,
) -> LaunchPlan {
    if candidates.is_empty() {
        return LaunchPlan {
            content_path: content_path.to_path_buf(),
            content_extension,
            candidates,
            selected_core: None,
            decision: LaunchDecisionKind::NoCore,
            reason: "no compatible core registered for content extension".to_string(),
        };
    }

    if let Some(preferred_core) = preferred_core {
        if let Some(core) = candidates
            .iter()
            .find(|core| paths_equal(&core.path, preferred_core))
            .cloned()
        {
            return LaunchPlan {
                content_path: content_path.to_path_buf(),
                content_extension,
                candidates,
                selected_core: Some(core.path),
                decision: LaunchDecisionKind::Selected,
                reason: "preferred core mapping selected".to_string(),
            };
        }
    }

    if candidates.len() == 1 {
        return LaunchPlan::selected(
            content_path.to_path_buf(),
            content_extension,
            candidates.into_iter().next().unwrap(),
        );
    }

    LaunchPlan {
        content_path: content_path.to_path_buf(),
        content_extension,
        candidates,
        selected_core: None,
        decision: LaunchDecisionKind::NeedsCoreChoice,
        reason: "multiple compatible cores found; user must choose".to_string(),
    }
}

fn extension_for_content(path: &Path) -> String {
    let ext = path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .trim_start_matches('.')
        .to_lowercase();
    if ext.is_empty() {
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_lowercase()
    } else {
        ext
    }
}

fn paths_equal(left: &Path, right: &Path) -> bool {
    if left == right {
        return true;
    }
    match (left.canonicalize(), right.canonicalize()) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn core(path: &str, extensions: &[&str]) -> CoreInfo {
        CoreInfo {
            path: PathBuf::from(path),
            display_name: path.to_string(),
            supported_extensions: extensions.iter().map(|s| s.to_string()).collect(),
            ..CoreInfo::default()
        }
    }

    #[test]
    fn asks_for_choice_when_multiple_cores_support_same_extension() {
        let mut list = CoreInfoList::new();
        list.cores = vec![
            core("/cores/a.dylib", &["gba"]),
            core("/cores/b.dylib", &["gba"]),
        ];
        list.rebuild_indexes();
        let settings = Settings::new();
        let mut launcher = LaunchManager::new();

        let plan =
            launcher.plan_content_launch(Path::new("/roms/game.gba"), &list, &settings, None);

        assert_eq!(plan.decision, LaunchDecisionKind::NeedsCoreChoice);
        assert_eq!(plan.candidates.len(), 2);
        assert!(plan.selected_core.is_none());
    }

    #[test]
    fn selects_preferred_core_mapping() {
        let mut list = CoreInfoList::new();
        list.cores = vec![
            core("/cores/a.dylib", &["gba"]),
            core("/cores/b.dylib", &["gba"]),
        ];
        list.rebuild_indexes();
        let mut settings = Settings::new();
        settings.set_preferred_core_for_extension("gba", Path::new("/cores/b.dylib"));
        let mut launcher = LaunchManager::new();

        let plan =
            launcher.plan_content_launch(Path::new("/roms/game.gba"), &list, &settings, None);

        assert_eq!(plan.decision, LaunchDecisionKind::Selected);
        assert_eq!(plan.selected_core, Some(PathBuf::from("/cores/b.dylib")));
    }
}
