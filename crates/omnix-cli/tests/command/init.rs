use std::path::Path;

use crate::command::core::om;
use assert_cmd::Command;
use nix_rs::{command::NixCmd, flake::url::FlakeUrl};
use omnix_init::registry::BUILTIN_REGISTRY;
use predicates::str::contains;

/// `om init` runs and successfully initializes a template
#[tokio::test]
async fn om_init() -> anyhow::Result<()> {
    for test in om_init_tests() {
        test.run_test().await?;
    }
    Ok(())
}

fn om_init_tests() -> Vec<OmInitTest> {
    let registry = BUILTIN_REGISTRY.clone();
    let lookup = |name: &str| registry.0.get(name).cloned().unwrap();
    vec![
        OmInitTest {
            template_name: lookup("haskell-template"),
            default_params: r#"{"package-name": "foo", "author": "John", "vscode": false }"#,
            asserts: Asserts {
                out_dir: PathAsserts {
                    exists: vec![".github/workflows/ci.yaml"],
                    not_exists: vec![".vscode"],
                },
                nix_run_output_contains: Some("from foo"),
                ..Default::default()
            },
        },
        OmInitTest {
            template_name: lookup("rust-nix-template"),
            default_params: r#"{"package-name": "qux", "author": "John", "author-email": "john@example.com" }"#,
            asserts: Asserts {
                out_dir: PathAsserts {
                    exists: vec![
                        "Cargo.toml",
                        "flake.nix",
                        ".github/workflows/ci.yml",
                        ".vscode",
                    ],
                    not_exists: vec!["nix/modules/template.nix"],
                },
                nix_run_output_contains: Some("from qux"),
                ..Default::default()
            },
        },
        OmInitTest {
            template_name: lookup("nixos-unified-template").with_attr("home"),
            default_params: r#"{"username": "john", "git-email": "jon@ex.com", "git-name": "John", "neovim": true }"#,
            asserts: Asserts {
                out_dir: PathAsserts {
                    exists: vec!["modules/home/neovim/default.nix"],
                    not_exists: vec![".github/workflows"],
                },
                nix_build_result: Some((
                    "homeConfigurations.john.activationPackage".to_string(),
                    PathAsserts {
                        exists: vec!["home-path/bin/nvim"],
                        not_exists: vec!["home-path/bin/vim"],
                    },
                )),
                ..Default::default()
            },
        },
    ]
}

/// A test for a single template
struct OmInitTest {
    /// The template name to pass to `om init`
    template_name: FlakeUrl,
    /// The --default-params to pass to `om init`
    default_params: &'static str,
    /// Various assertions to make after running `om init`
    asserts: Asserts,
}

impl OmInitTest {
    /// Run this test on a temporary directory
    async fn run_test(&self) -> anyhow::Result<()> {
        let temp_dir = assert_fs::TempDir::new().unwrap();
        om()?
            .args([
                "init",
                "-o",
                &temp_dir.to_string_lossy(),
                &self.template_name,
                "--non-interactive",
                "--params",
                self.default_params,
            ])
            .assert()
            .success();

        // Recursively print the contents of temp_dir to debug test failures
        let paths = omnix_common::fs::find_paths(&temp_dir).await?;
        println!(
            "[{}] Paths in temp_dir {}:",
            self.template_name,
            temp_dir.path().display()
        );
        for path in paths {
            println!("  {}", path.display());
        }

        // Run assertion tests
        self.asserts.assert(&temp_dir).await?;

        temp_dir.close().unwrap();
        Ok(())
    }
}

#[derive(Default)]
struct Asserts {
    out_dir: PathAsserts,
    /// The output of `nix run` should contain this string
    nix_run_output_contains: Option<&'static str>,
    /// The store path built by `nix build .#attr` should contain these paths
    nix_build_result: Option<(String, PathAsserts)>,
}

impl Asserts {
    async fn assert(&self, dir: &Path) -> anyhow::Result<()> {
        self.out_dir.assert(dir);

        if let Some(nix_run_output_contains) = self.nix_run_output_contains {
            Command::new("nix")
                .arg("run")
                .arg(FlakeUrl::from(dir).to_string())
                .assert()
                .success()
                .stdout(contains(nix_run_output_contains));
        }

        if let Some((attr, nix_build_result)) = &self.nix_build_result {
            let paths = nix_rs::flake::command::build(
                &NixCmd::default(),
                FlakeUrl::from(dir).with_attr(attr),
            )
            .await?;
            assert_matches!(paths.first().and_then(|v| v.first_output()), Some(path) => {
                nix_build_result.assert(path);
            });
        }

        Ok(())
    }
}

#[derive(Default)]
struct PathAsserts {
    // Assert that these paths exist
    exists: Vec<&'static str>,
    // Assert that these paths do not exist
    not_exists: Vec<&'static str>,
}

impl PathAsserts {
    fn assert(&self, dir: &Path) {
        for path in &self.exists {
            assert!(
                dir.join(path).exists(),
                "Expected path to exist: {:?} (under {:?})",
                path,
                dir,
            );
        }
        for path in &self.not_exists {
            assert!(
                !dir.join(path).exists(),
                "Expected path to not exist: {:?}",
                path,
            );
        }
    }
}
