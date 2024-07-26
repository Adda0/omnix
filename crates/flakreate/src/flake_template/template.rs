use colored::Colorize;
use core::fmt;
use std::{
    collections::BTreeMap,
    fmt::{Display, Formatter},
};

use nix_rs::{
    command::{NixCmd, NixCmdError},
    flake::url::FlakeUrl,
};
use serde::{Deserialize, Serialize};

use super::{config::FlakeTemplateConfig, fileop::FileOp};

/// A Nix flake template
///
/// Defined per [this definition](https://nix.dev/manual/nix/2.22/command-ref/new-cli/nix3-flake-init#template-definitions) in the flake.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct FlakeTemplate {
    #[serde(skip_deserializing)]
    pub name: String,

    pub description: String,

    pub path: String,

    #[serde(rename = "welcomeText")]
    pub welcome_text: Option<String>,

    #[serde(skip_deserializing)]
    pub config: FlakeTemplateConfig,
}

impl Display for FlakeTemplate {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.config.tags.is_empty() {
            write!(f, "{}", self.name)
        } else {
            write!(
                f,
                "{:<20} {}",
                self.name,
                self.config.tags.join(", ").dimmed()
            )
        }
    }
}

impl FlakeTemplate {
    pub fn prompt_replacements(&self) -> anyhow::Result<Vec<Vec<FileOp>>> {
        self.config
            .params
            .iter()
            .map(|param| param.prompt_value())
            .collect()
    }
}

/// Fetch the templates defined in a flake
pub async fn fetch(url: &FlakeUrl) -> Result<Vec<FlakeTemplate>, NixCmdError> {
    let nixcmd = NixCmd::get().await;
    let mut templates = nix_rs::flake::eval::nix_eval_attr_json::<BTreeMap<String, FlakeTemplate>>(
        nixcmd,
        &url.with_attr("templates"),
    )
    .await?
    .unwrap_or_default();
    let templates_config = nix_rs::flake::eval::nix_eval_attr_json::<
        BTreeMap<String, FlakeTemplateConfig>,
    >(nixcmd, &url.with_attr("om.templates"))
    .await?
    .unwrap_or_default();
    for (name, template) in templates.iter_mut() {
        // Set 'name' field in each template
        template.name.clone_from(name);
        // Pull in `om.templates` configuration
        template.config = templates_config.get(name).cloned().unwrap_or_default();
    }
    Ok(templates.values().cloned().collect())
}