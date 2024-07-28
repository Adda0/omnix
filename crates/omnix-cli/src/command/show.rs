use std::{collections::BTreeMap, io::IsTerminal};

use anyhow::Context;
use clap::Parser;
use colored::Colorize;
use nix_rs::{
    command::NixCmd,
    config::NixConfig,
    flake::{outputs::Val, url::FlakeUrl, Flake},
};
use tabled::{
    settings::{location::ByColumnName, Color, Modify, Style},
    Table, Tabled,
};

/// Inspect a flake
#[derive(Parser, Debug)]
pub struct ShowConfig {
    /// The flake to show outputs for
    #[arg(name = "FLAKE")]
    pub flake_url: FlakeUrl,
}

/// Tabular representation of a set of flake outputs (eg: `packages.*`)
pub struct FlakeOutputTable {
    /// Rows of the table
    pub rows: Vec<Row>,
    /// Title of the table
    pub title: String,
    /// Command to run the outputs in the `name` column
    pub command: String,
}

impl FlakeOutputTable {
    /// Convert the table to a [Table] struct
    fn to_tabled(&self) -> Table {
        let mut table = Table::new(&self.rows);
        table.with(Style::rounded());
        if std::io::stdout().is_terminal() {
            table.with(Modify::new(ByColumnName::new("name")).with(Color::BOLD));
        };
        table
    }

    /// Print the table to stdout
    pub fn print(&self) {
        if self.rows.is_empty() {
            return;
        }
        print!("{}", self.title.blue().bold());
        println!(" ({})", self.command.green().bold());

        println!("{}", self.to_tabled());
        println!();
    }
}

/// Row in a [FlakeOutputTable]
#[derive(Tabled)]
pub struct Row {
    /// Name of the output
    pub name: String,
    /// Description of the output
    pub description: String,
}

impl Row {
    /// Convert a [BTreeMap] to a vector of [Row]s
    pub fn vec_from_btreemap(map: BTreeMap<String, Val>) -> Vec<Row> {
        map.into_iter()
            .map(|(name, val)| Row {
                name,
                description: val.description.unwrap_or_else(|| "N/A".to_string()),
            })
            .collect()
    }
}

impl ShowConfig {
    pub async fn run(&self) -> anyhow::Result<()> {
        let nix_cmd = NixCmd::get().await;
        let nix_config = NixConfig::get().await.as_ref()?;
        let flake = Flake::from_nix(nix_cmd, nix_config, self.flake_url.clone())
            .await
            .with_context(|| "Unable to fetch flake")?;

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.packages),
            title: "📦 Packages".to_string(),
            command: format!("nix build {}#<name>", self.flake_url),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.devshells),
            title: "🐚 Devshells".to_string(),
            command: format!("nix develop {}#<name>", self.flake_url),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.apps),
            title: "🚀 Apps".to_string(),
            command: format!("nix run {}#<name>", self.flake_url),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.checks),
            title: "🔍 Checks".to_string(),
            command: "nix flake check".to_string(),
        }
        .print();

        FlakeOutputTable {
            rows: Row::vec_from_btreemap(flake.schema.nixos_configurations),
            title: "🐧 NixOS Configurations".to_string(),
            command: "nixos-rebuild switch --flake .#<name>".to_string(),
        }
        .print();

        Ok(())
    }
}
