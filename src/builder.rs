//! Lightweight public builders for assembling showcase apps and pages.

use crate::schema::{PageSpec, SectionSpec};
use crate::showcase::{ShowcaseApp, ShowcaseCopy, ShowcaseScreen};
use crate::{
    components::ContentControl, executor::ActionRegistry, host::RuntimeHost,
    runtime::OperationSource,
};
use std::collections::{HashMap, HashSet};

/// Start a page definition with a title.
pub fn page(title: impl Into<String>) -> PageSpec {
    PageSpec::new(title)
}

/// Start a section definition with a title.
pub fn section(title: impl Into<String>) -> SectionSpec {
    SectionSpec::new(title)
}

/// Wrap a page definition into a menu-visible screen.
pub fn screen(title: impl Into<String>, page: PageSpec) -> ShowcaseScreen {
    ShowcaseScreen::from_page(title, page)
}

/// Top-level application specification.
///
/// This is the preferred public entry point for assembling a runnable app:
/// title copy, control help text, and the set of menu screens all live here.
pub struct AppSpec {
    title_text: String,
    status_controls: String,
    screens: Vec<ShowcaseScreen>,
    shell_actions: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppValidationError {
    DuplicateFieldId(String),
    MissingResultTarget {
        source_field: String,
        target_id: String,
    },
    InvalidResultTarget {
        source_field: String,
        target_id: String,
    },
    UnknownRegisteredAction {
        source_field: String,
        action: String,
    },
}

impl std::fmt::Display for AppValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DuplicateFieldId(id) => write!(f, "duplicate field id: {id}"),
            Self::MissingResultTarget {
                source_field,
                target_id,
            } => write!(
                f,
                "field {source_field} references missing result target id: {target_id}"
            ),
            Self::InvalidResultTarget {
                source_field,
                target_id,
            } => write!(
                f,
                "field {source_field} references non-log result target id: {target_id}"
            ),
            Self::UnknownRegisteredAction {
                source_field,
                action,
            } => write!(
                f,
                "field {source_field} references unknown registered action: {action}"
            ),
        }
    }
}

impl std::error::Error for AppValidationError {}

impl AppSpec {
    /// Create an empty application definition.
    pub fn new() -> Self {
        Self {
            title_text: String::new(),
            status_controls: String::new(),
            screens: Vec::new(),
            shell_actions: Vec::new(),
        }
    }

    /// Set the left-top title panel text.
    pub fn title_text(mut self, title_text: impl Into<String>) -> Self {
        self.title_text = title_text.into();
        self
    }

    /// Set the right-top help/controls text.
    pub fn status_controls(mut self, status_controls: impl Into<String>) -> Self {
        self.status_controls = status_controls.into();
        self
    }

    /// Append a single screen.
    pub fn screen(mut self, screen: ShowcaseScreen) -> Self {
        self.screens.push(screen);
        self
    }

    /// Append multiple screens.
    pub fn screens(mut self, screens: Vec<ShowcaseScreen>) -> Self {
        self.screens.extend(screens);
        self
    }

    /// Register a named shell action that can be referenced by config.
    pub fn shell_action(mut self, name: impl Into<String>, command: impl Into<String>) -> Self {
        self.shell_actions.push((name.into(), command.into()));
        self
    }

    pub fn validate(&self) -> Result<(), AppValidationError> {
        let actions = self
            .shell_actions
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<HashSet<_>>();
        self.validate_with_actions(&actions)
    }

    pub fn validate_with_registry(
        &self,
        registry: &ActionRegistry,
    ) -> Result<(), AppValidationError> {
        let mut actions = self
            .shell_actions
            .iter()
            .map(|(name, _)| name.clone())
            .collect::<HashSet<_>>();
        for name in self.registered_action_names() {
            if registry.has_action(&name) {
                actions.insert(name);
            }
        }
        self.validate_with_actions(&actions)
    }

    fn validate_with_actions(&self, actions: &HashSet<String>) -> Result<(), AppValidationError> {
        for screen in &self.screens {
            let mut ids = HashMap::<String, bool>::new();
            for section in &screen.content.sections {
                for block in &section.blocks {
                    if let Some(id) = &block.id {
                        let is_log = matches!(block.control, ContentControl::LogOutput(_));
                        if ids.insert(id.clone(), is_log).is_some() {
                            return Err(AppValidationError::DuplicateFieldId(id.clone()));
                        }
                    }
                }
            }

            for section in &screen.content.sections {
                for block in &section.blocks {
                    let source_field = block
                        .id
                        .clone()
                        .unwrap_or_else(|| format!("{}:{}", screen.title, block.label));

                    if let Some(operation) = &block.operation {
                        if let Some(target_id) = &operation.result_target {
                            match ids.get(target_id) {
                                None => {
                                    return Err(AppValidationError::MissingResultTarget {
                                        source_field,
                                        target_id: target_id.clone(),
                                    });
                                }
                                Some(false) => {
                                    return Err(AppValidationError::InvalidResultTarget {
                                        source_field,
                                        target_id: target_id.clone(),
                                    });
                                }
                                Some(true) => {}
                            }
                        }

                        if let OperationSource::RegisteredAction(action) = &operation.source {
                            if !actions.contains(action) {
                                return Err(AppValidationError::UnknownRegisteredAction {
                                    source_field,
                                    action: action.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn registered_action_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        for screen in &self.screens {
            for section in &screen.content.sections {
                for block in &section.blocks {
                    if let Some(operation) = &block.operation {
                        if let OperationSource::RegisteredAction(action) = &operation.source {
                            names.push(action.clone());
                        }
                    }
                }
            }
        }
        names
    }

    /// Materialize the spec into a runnable showcase application.
    pub fn into_showcase_app(self) -> ShowcaseApp {
        self.into_showcase_app_with_registry(ActionRegistry::new())
    }

    pub fn into_showcase_app_with_registry(self, mut registry: ActionRegistry) -> ShowcaseApp {
        for (name, command) in self.shell_actions {
            registry.register_shell_action(name, command);
        }
        ShowcaseApp::with_registry(
            ShowcaseCopy {
                title_text: self.title_text,
                status_controls: self.status_controls,
            },
            self.screens,
            registry,
        )
    }

    pub fn into_showcase_app_with_host(self, mut host: RuntimeHost) -> ShowcaseApp {
        for (name, command) in self.shell_actions {
            host.register_shell_action(name, command);
        }
        ShowcaseApp::with_host(
            ShowcaseCopy {
                title_text: self.title_text,
                status_controls: self.status_controls,
            },
            self.screens,
            host,
        )
    }

    pub fn try_into_showcase_app(self) -> Result<ShowcaseApp, AppValidationError> {
        self.validate()?;
        Ok(self.into_showcase_app())
    }

    pub fn try_into_showcase_app_with_registry(
        self,
        registry: ActionRegistry,
    ) -> Result<ShowcaseApp, AppValidationError> {
        self.validate_with_registry(&registry)?;
        Ok(self.into_showcase_app_with_registry(registry))
    }

    pub fn try_into_showcase_app_with_host(
        self,
        host: RuntimeHost,
    ) -> Result<ShowcaseApp, AppValidationError> {
        self.validate_with_registry(&host.action_registry())?;
        Ok(self.into_showcase_app_with_host(host))
    }
}

impl Default for AppSpec {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{page, screen, section, AppSpec, AppValidationError};
    use crate::executor::{ActionOutcome, ActionRegistry};
    use crate::host::RuntimeHost;
    use crate::schema::FieldSpec;

    #[test]
    fn page_builder_supports_chained_sections_and_fields() {
        let page = page("Workspace").section(
            section("Main")
                .field(FieldSpec::text_input("项目名", "tui01", "输入项目名"))
                .field(FieldSpec::toggle("启用缓存", true)),
        );

        let runtime = page.materialize();
        assert_eq!(runtime.sections.len(), 1);
        assert_eq!(runtime.sections[0].fields.len(), 2);
    }

    #[test]
    fn app_spec_collects_screens() {
        let app = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .screen(screen("One", page("One")))
            .into_showcase_app();

        assert_eq!(app.active_screen(), 0);
    }

    #[test]
    fn app_spec_builds_showcase_app() {
        let app = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .screen(screen("One", page("One")))
            .into_showcase_app();

        assert_eq!(app.active_screen(), 0);
    }

    #[test]
    fn app_spec_validation_rejects_unknown_registered_action() {
        let err = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .screen(screen(
                "One",
                page("One").section(
                    section("Actions").field(
                        FieldSpec::refresh_button("刷新", "刷新")
                            .with_registered_action("missing_action"),
                    ),
                ),
            ))
            .validate()
            .unwrap_err();

        assert!(matches!(
            err,
            AppValidationError::UnknownRegisteredAction { .. }
        ));
    }

    #[test]
    fn app_spec_validation_rejects_missing_result_target() {
        let err = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .shell_action("refresh_workspace", "printf 'ok\\n'")
            .screen(screen(
                "One",
                page("One").section(
                    section("Actions").field(
                        FieldSpec::refresh_button("刷新", "刷新")
                            .with_registered_action("refresh_workspace")
                            .with_result_target("missing_log"),
                    ),
                ),
            ))
            .validate()
            .unwrap_err();

        assert!(matches!(
            err,
            AppValidationError::MissingResultTarget { .. }
        ));
    }

    #[test]
    fn app_spec_validation_allows_same_field_id_on_different_screens() {
        let result = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .screen(screen(
                "One",
                page("One").section(
                    section("Fields")
                        .field(FieldSpec::text_input("项目名", "a", "输入").with_id("shared_id"))
                        .field(
                            FieldSpec::log_output("日志", "ready")
                                .with_id("screen_log")
                                .with_height_units(4),
                        )
                        .field(
                            FieldSpec::refresh_button("刷新", "刷新")
                                .with_registered_action("refresh_workspace")
                                .with_result_target("screen_log"),
                        ),
                ),
            ))
            .screen(screen(
                "Two",
                page("Two").section(
                    section("Fields")
                        .field(FieldSpec::text_input("项目名", "b", "输入").with_id("shared_id"))
                        .field(
                            FieldSpec::log_output("日志", "ready")
                                .with_id("screen_log")
                                .with_height_units(4),
                        )
                        .field(
                            FieldSpec::refresh_button("刷新", "刷新")
                                .with_registered_action("refresh_workspace")
                                .with_result_target("screen_log"),
                        ),
                ),
            ))
            .shell_action("refresh_workspace", "printf 'ok\\n'")
            .validate();

        assert!(result.is_ok());
    }

    #[test]
    fn app_spec_with_runtime_registry_accepts_registered_handler() {
        let mut registry = ActionRegistry::new();
        registry.register_action_handler("refresh_workspace", |_| async {
            ActionOutcome::success("ok")
        });

        let result = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .screen(screen(
                "One",
                page("One").section(
                    section("Actions").field(
                        FieldSpec::refresh_button("刷新", "刷新")
                            .with_registered_action("refresh_workspace"),
                    ),
                ),
            ))
            .try_into_showcase_app_with_registry(registry);

        assert!(result.is_ok());
    }

    #[test]
    fn app_spec_with_runtime_host_accepts_registered_handler() {
        let mut host = RuntimeHost::new();
        host.register_action_handler("refresh_workspace", |_| async {
            ActionOutcome::success("ok")
        });

        let result = AppSpec::new()
            .title_text("Demo")
            .status_controls("Controls")
            .screen(screen(
                "One",
                page("One").section(
                    section("Actions").field(
                        FieldSpec::refresh_button("刷新", "刷新")
                            .with_registered_action("refresh_workspace"),
                    ),
                ),
            ))
            .try_into_showcase_app_with_host(host);

        assert!(result.is_ok());
    }
}
