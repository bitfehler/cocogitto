use anyhow::Result;
use cmd_lib::run_cmd;
use cocogitto::settings::Settings;
use cocogitto::{conventional::version::IncrementCommand, CocoGitto};
use sealed_test::prelude::*;
use speculoos::prelude::*;

use crate::helpers::*;

#[sealed_test]
fn bump_ok() -> Result<()> {
    // Arrange
    git_init()?;
    git_commit("chore: first commit")?;
    git_commit("feat: add a feature commit")?;
    git_tag("1.0.0")?;
    git_commit("feat: add another feature commit")?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_ok();
    assert_latest_tag("1.1.0")?;
    Ok(())
}

#[sealed_test]
fn monorepo_bump_ok() -> Result<()> {
    // Arrange
    let mut settings = Settings {
        ..Default::default()
    };

    init_monorepo(&mut settings)?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_monorepo_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_ok();
    assert_tag_exists("0.1.0")?;
    assert_tag_exists("one-0.1.0")?;
    Ok(())
}

#[sealed_test]
fn monorepo_bump_manual_ok() -> Result<()> {
    // Arrange
    let mut settings = Settings {
        ..Default::default()
    };

    init_monorepo(&mut settings)?;
    run_cmd!(
        git tag "one-0.1.0";
    )?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_monorepo_version(IncrementCommand::Major, None, None, false);

    // Assert
    assert_that!(result).is_ok();
    assert_tag_exists("1.0.0")?;
    Ok(())
}

#[sealed_test]
fn monorepo_with_tag_prefix_bump_ok() -> Result<()> {
    // Arrange
    let mut settings = Settings {
        tag_prefix: Some("v".to_string()),
        ..Default::default()
    };

    init_monorepo(&mut settings)?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_monorepo_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_ok();
    assert_tag_exists("v0.1.0")?;
    assert_tag_exists("one-v0.1.0")?;
    Ok(())
}

#[sealed_test]
fn package_bump_ok() -> Result<()> {
    // Arrange
    let mut settings = Settings {
        ..Default::default()
    };

    init_monorepo(&mut settings)?;
    let package = settings.packages.get("one").unwrap();
    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_package_version(
        ("one", package),
        IncrementCommand::AutoPackage("one".to_string()),
        None,
        None,
        false,
    );

    // Assert
    assert_that!(result).is_ok();
    assert_tag_does_not_exist("0.1.0")?;
    assert_tag_exists("one-0.1.0")?;
    Ok(())
}

#[sealed_test]
fn should_fallback_to_0_0_0_when_there_is_no_tag() -> Result<()> {
    // Arrange
    git_init()?;
    git_commit("chore: first commit")?;
    git_commit("feat: add a feature commit")?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_ok();
    assert_latest_tag("0.1.0")?;
    Ok(())
}

// FIXME: Failing on non compliant tag should be configurable
//  until it's implemented we will ignore non compliant tags
// #[sealed_test]
// fn should_fail_when_latest_tag_is_not_semver_compliant() -> Result<()> {
//     // Arrange
//     git_init()?;
//     git_commit("chore: first commit")?;
//     git_commit("feat: add a feature commit")?;
//     git_tag("toto")?;
//     git_commit("feat: add another feature commit")?;
//
//     let mut cocogitto = CocoGitto::get()?;
//
//     // Act
//     let result = cocogitto.create_version(VersionIncrement::Auto, None, None, false);
//     let error = result.unwrap_err().to_string();
//     let error = error.as_str();
//
//     // Assert
//     assert_that!(error).is_equal_to(indoc!(
//         "
//         tag `toto` is not SemVer compliant
//         \tcause: unexpected character 't' while parsing major version number
//         "
//     ));
//     Ok(())
// }

#[sealed_test]
fn bump_with_whitelisted_branch_ok() -> Result<()> {
    // Arrange
    let settings = r#"branch_whitelist = [ "master" ]"#;

    git_init()?;
    run_cmd!(
        echo $settings > cog.toml;
        git add .;
    )?;

    git_commit("chore: first commit")?;
    git_commit("feat: add a feature commit")?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_ok();

    Ok(())
}

#[sealed_test]
fn bump_with_whitelisted_branch_fails() -> Result<()> {
    // Arrange
    let settings = r#"branch_whitelist = [ "main" ]"#;

    git_init()?;
    run_cmd!(
        echo $settings > cog.toml;
        git add .;
    )?;

    git_commit("chore: first commit")?;
    git_commit("feat: add a feature commit")?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result.unwrap_err().to_string()).is_equal_to(
        "No patterns matched in [\"main\"] for branch 'master', bump is not allowed".to_string(),
    );

    Ok(())
}

#[sealed_test]
fn bump_with_whitelisted_branch_pattern_ok() -> Result<()> {
    // Arrange
    let settings = r#"branch_whitelist = [ "main", "release/**" ]"#;

    git_init()?;
    run_cmd!(
        echo $settings > cog.toml;
        git add .;
    )?;

    git_commit("chore: first commit")?;
    git_commit("feat: add a feature commit")?;

    run_cmd!(git checkout -b release/1.0.0;)?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_ok();

    Ok(())
}

#[sealed_test]
fn bump_with_whitelisted_branch_pattern_err() -> Result<()> {
    // Arrange
    let settings = r#"branch_whitelist = [ "release/**" ]"#;

    git_init()?;
    run_cmd!(
        echo $settings > cog.toml;
        git add .;
    )?;

    git_commit("chore: first commit")?;
    git_commit("feat: add a feature commit")?;

    let mut cocogitto = CocoGitto::get()?;

    // Act
    let result = cocogitto.create_version(IncrementCommand::Auto, None, None, false);

    // Assert
    assert_that!(result).is_err();

    Ok(())
}
