//! Serializable external configuration format.

use crate::builder::{page, screen, section, AppSpec};
use crate::schema::FieldSpec;
use crate::showcase::ShowcaseApp;
use mlua::LuaSerdeExt;
use serde::{Deserialize, Serialize};
use std::{fs, io, path::Path};

#[derive(Debug)]
pub enum ConfigLoadError {
    Io(io::Error),
    Json(serde_json::Error),
    Yaml(serde_yaml::Error),
    Lua(mlua::Error),
    UnsupportedExtension(String),
}

impl std::fmt::Display for ConfigLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(err) => write!(f, "failed to read config file: {err}"),
            Self::Json(err) => write!(f, "failed to parse JSON config: {err}"),
            Self::Yaml(err) => write!(f, "failed to parse YAML config: {err}"),
            Self::Lua(err) => write!(f, "failed to parse Lua config: {err}"),
            Self::UnsupportedExtension(ext) => {
                write!(f, "unsupported config extension: {ext}")
            }
        }
    }
}

impl std::error::Error for ConfigLoadError {}

impl From<io::Error> for ConfigLoadError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<mlua::Error> for ConfigLoadError {
    fn from(value: mlua::Error) -> Self {
        Self::Lua(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub title_text: String,
    pub status_controls: String,
    #[serde(default)]
    pub screens: Vec<ScreenConfig>,
}

impl AppConfig {
    /// Parse configuration from a JSON string.
    pub fn from_json_str(input: &str) -> serde_json::Result<Self> {
        serde_json::from_str(input)
    }

    /// Parse configuration from a YAML string.
    pub fn from_yaml_str(input: &str) -> serde_yaml::Result<Self> {
        serde_yaml::from_str(input)
    }

    /// Parse configuration from a Lua chunk.
    ///
    /// The chunk must return a table matching [`AppConfig`].
    pub fn from_lua_str(input: &str) -> Result<Self, ConfigLoadError> {
        let lua = mlua::Lua::new();
        let value = lua.load(input).eval::<mlua::Value>()?;
        lua.from_value(value).map_err(ConfigLoadError::Lua)
    }

    /// Load configuration from a JSON file.
    pub fn from_json_file(path: impl AsRef<Path>) -> Result<Self, ConfigLoadError> {
        let content = fs::read_to_string(path)?;
        Self::from_json_str(&content).map_err(ConfigLoadError::Json)
    }

    /// Load configuration from a YAML file.
    pub fn from_yaml_file(path: impl AsRef<Path>) -> Result<Self, ConfigLoadError> {
        let content = fs::read_to_string(path)?;
        Self::from_yaml_str(&content).map_err(ConfigLoadError::Yaml)
    }

    /// Load configuration from a Lua file.
    pub fn from_lua_file(path: impl AsRef<Path>) -> Result<Self, ConfigLoadError> {
        let content = fs::read_to_string(path)?;
        Self::from_lua_str(&content)
    }

    /// Load configuration based on file extension.
    ///
    /// Supported extensions:
    /// - `.json`
    /// - `.yaml`
    /// - `.yml`
    /// - `.lua`
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self, ConfigLoadError> {
        let path = path.as_ref();
        match path.extension().and_then(|ext| ext.to_str()) {
            Some("json") => Self::from_json_file(path),
            Some("yaml") | Some("yml") => Self::from_yaml_file(path),
            Some("lua") => Self::from_lua_file(path),
            Some(ext) => Err(ConfigLoadError::UnsupportedExtension(ext.to_string())),
            None => Err(ConfigLoadError::UnsupportedExtension(String::new())),
        }
    }

    pub fn into_app_spec(self) -> AppSpec {
        self.screens.into_iter().fold(
            AppSpec::new()
                .title_text(self.title_text)
                .status_controls(self.status_controls),
            |spec, screen_config| spec.screen(screen_config.into_screen()),
        )
    }

    pub fn into_showcase_app(self) -> ShowcaseApp {
        self.into_app_spec().into_showcase_app()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScreenConfig {
    pub title: String,
    pub page: PageConfig,
}

impl ScreenConfig {
    pub fn into_screen(self) -> crate::showcase::ShowcaseScreen {
        screen(self.title, self.page.into_page_spec())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PageConfig {
    pub title: String,
    #[serde(default)]
    pub sections: Vec<SectionConfig>,
}

impl PageConfig {
    pub fn into_page_spec(self) -> crate::schema::PageSpec {
        self.sections
            .into_iter()
            .fold(page(self.title), |page_spec, section_config| {
                page_spec.section(section_config.into_section_spec())
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SectionConfig {
    pub title: String,
    #[serde(default)]
    pub fields: Vec<FieldConfig>,
}

impl SectionConfig {
    pub fn into_section_spec(self) -> crate::schema::SectionSpec {
        self.fields
            .into_iter()
            .fold(section(self.title), |section_spec, field_config| {
                section_spec.field(field_config.into_field_spec())
            })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FieldConfig {
    pub label: String,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default = "default_height_units")]
    pub height_units: u16,
    pub control: FieldControlConfig,
    #[serde(default)]
    pub operation: Option<OperationConfig>,
}

impl FieldConfig {
    pub fn into_field_spec(self) -> FieldSpec {
        let mut field = self.control.into_field_spec(self.label);
        if let Some(id) = self.id {
            field = field.with_id(id);
        }
        field = field.with_height_units(self.height_units);
        if let Some(operation) = self.operation {
            field = operation.apply(field);
        }
        field
    }
}

fn default_height_units() -> u16 {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum FieldControlConfig {
    TextInput {
        value: String,
        placeholder: String,
    },
    NumberInput {
        value: String,
        placeholder: String,
    },
    Select {
        options: Vec<String>,
        selected: usize,
    },
    Toggle {
        on: bool,
    },
    ActionButton {
        button_label: String,
    },
    RefreshButton {
        button_label: String,
    },
    StaticData {
        value: String,
    },
    DynamicData {
        value: String,
    },
    LogOutput {
        content: String,
    },
}

impl FieldControlConfig {
    fn into_field_spec(self, label: String) -> FieldSpec {
        match self {
            Self::TextInput { value, placeholder } => {
                FieldSpec::text_input(label, value, placeholder)
            }
            Self::NumberInput { value, placeholder } => {
                FieldSpec::number_input(label, value, placeholder)
            }
            Self::Select { options, selected } => FieldSpec::select(label, options, selected),
            Self::Toggle { on } => FieldSpec::toggle(label, on),
            Self::ActionButton { button_label } => FieldSpec::action_button(label, button_label),
            Self::RefreshButton { button_label } => FieldSpec::refresh_button(label, button_label),
            Self::StaticData { value } => FieldSpec::static_data(label, value),
            Self::DynamicData { value } => FieldSpec::dynamic_data(label, value),
            Self::LogOutput { content } => FieldSpec::log_output(label, content),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum OperationConfig {
    Shell {
        command: String,
        #[serde(default)]
        result_target: Option<String>,
    },
    RegisteredAction {
        name: String,
        #[serde(default)]
        result_target: Option<String>,
    },
    SimulatedSuccess {
        duration_ms: u64,
        #[serde(default)]
        result_target: Option<String>,
    },
    SimulatedFailure {
        duration_ms: u64,
        #[serde(default)]
        result_target: Option<String>,
    },
}

impl OperationConfig {
    fn apply(self, field: FieldSpec) -> FieldSpec {
        match self {
            Self::Shell {
                command,
                result_target,
            } => {
                let field = field.with_shell_command(command);
                if let Some(target) = result_target {
                    field.with_result_target(target)
                } else {
                    field
                }
            }
            Self::RegisteredAction {
                name,
                result_target,
            } => {
                let field = field.with_registered_action(name);
                if let Some(target) = result_target {
                    field.with_result_target(target)
                } else {
                    field
                }
            }
            Self::SimulatedSuccess {
                duration_ms,
                result_target,
            } => {
                let field = field.with_operation_success(duration_ms);
                if let Some(target) = result_target {
                    field.with_result_target(target)
                } else {
                    field
                }
            }
            Self::SimulatedFailure {
                duration_ms,
                result_target,
            } => {
                let field = field.with_operation_failure(duration_ms);
                if let Some(target) = result_target {
                    field.with_result_target(target)
                } else {
                    field
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{AppConfig, ConfigLoadError};
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_path(ext: &str) -> std::path::PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("tui01-config-{nanos}.{ext}"))
    }

    #[test]
    fn yaml_config_converts_to_app_spec() {
        let yaml = r#"
title_text: Demo
status_controls: Controls
screens:
  - title: Workspace
    page:
      title: Workspace
      sections:
        - title: Main
          fields:
            - label: 项目名
              control:
                type: text_input
                value: tui01
                placeholder: 输入项目名
            - label: 输出
              id: workspace_log
              height_units: 4
              control:
                type: log_output
                content: 等待结果
        - title: Actions
          fields:
            - label: 刷新
              control:
                type: refresh_button
                button_label: 刷新
              operation:
                kind: shell
                command: printf 'ok\n'
                result_target: workspace_log
"#;

        let config = AppConfig::from_yaml_str(yaml).unwrap();
        let app = config.into_showcase_app();
        assert_eq!(app.active_screen(), 0);
    }

    #[test]
    fn json_config_supports_select_and_toggle() {
        let json = r#"{
          "title_text": "Demo",
          "status_controls": "Controls",
          "screens": [
            {
              "title": "Settings",
              "page": {
                "title": "Settings",
                "sections": [
                  {
                    "title": "Main",
                    "fields": [
                      {
                        "label": "模式",
                        "control": {
                          "type": "select",
                          "options": ["a", "b"],
                          "selected": 1
                        }
                      },
                      {
                        "label": "启用",
                        "control": {
                          "type": "toggle",
                          "on": true
                        }
                      }
                    ]
                  }
                ]
              }
            }
          ]
        }"#;

        let config = AppConfig::from_json_str(json).unwrap();
        let spec = config.into_app_spec();
        let app = spec.into_showcase_app();
        assert_eq!(app.active_screen(), 0);
    }

    #[test]
    fn yaml_file_loads_successfully() {
        let path = temp_path("yaml");
        fs::write(
            &path,
            "title_text: Demo\nstatus_controls: Controls\nscreens: []\n",
        )
        .unwrap();

        let config = AppConfig::from_yaml_file(&path).unwrap();
        fs::remove_file(path).unwrap();

        assert_eq!(config.title_text, "Demo");
    }

    #[test]
    fn unsupported_extension_returns_error() {
        let err = AppConfig::from_file("config.txt").unwrap_err();
        assert!(matches!(err, ConfigLoadError::UnsupportedExtension(_)));
    }

    #[test]
    fn lua_config_converts_to_app_spec() {
        let lua = r#"
return {
  title_text = "Demo",
  status_controls = "Controls",
  screens = {
    {
      title = "Workspace",
      page = {
        title = "Workspace",
        sections = {
          {
            title = "Main",
            fields = {
              {
                label = "项目名",
                control = {
                  type = "text_input",
                  value = "tui01",
                  placeholder = "输入项目名",
                },
              },
              {
                label = "启用缓存",
                control = {
                  type = "toggle",
                  on = true,
                },
              },
            },
          },
        },
      },
    },
  },
}
"#;

        let config = AppConfig::from_lua_str(lua).unwrap();
        let app = config.into_showcase_app();
        assert_eq!(app.active_screen(), 0);
    }

    #[test]
    fn registered_action_config_maps_to_app() {
        let yaml = r#"
title_text: Demo
status_controls: Controls
screens:
  - title: Workspace
    page:
      title: Workspace
      sections:
        - title: Actions
          fields:
            - label: 刷新
              control:
                type: refresh_button
                button_label: 刷新
              operation:
                kind: registered_action
                name: refresh_workspace
"#;

        let config = AppConfig::from_yaml_str(yaml).unwrap();
        let app = config.into_showcase_app();
        assert_eq!(app.active_screen(), 0);
    }
}
