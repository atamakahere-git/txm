use std::process::Command;

#[test]
fn prints_usage_without_arguments() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
}

#[test]
fn boxes_simple_identifier() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg("x")
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "┌───┐\n│   │\n│ x │\n│   │\n└───┘\n"
    );
}

#[test]
fn boxes_wide_identifier() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg("你")
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "┌────┐\n│    │\n│ 你 │\n│    │\n└────┘\n"
    );
}

#[test]
fn boxes_adjacent_wide_identifiers() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg("你你")
        .output()
        .expect("failed to run txm");

    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "┌──────┐\n│      │\n│ 你你 │\n│      │\n└──────┘\n"
    );
}

#[test]
fn render_returns_raw_lines_for_simple_identifier() {
    let rendered = txm::render("x").expect("render failed");

    assert_eq!(rendered, "x\n");
}

#[test]
fn render_returns_error_for_unclosed_group() {
    assert!(txm::render("{x").is_err());
}

#[test]
fn render_returns_error_for_invalid_lexer_input() {
    assert!(txm::render("@").is_err());
}

#[test]
fn render_returns_error_for_unknown_matrix_environment() {
    assert!(txm::render(r"\begin{unknown}x\end{unknown}").is_err());
}

#[test]
fn render_returns_error_for_ragged_matrix() {
    assert!(txm::render(r"\begin{matrix}a&b\\c\end{matrix}").is_err());
}

#[test]
fn cli_reports_render_errors_without_panicking() {
    let output = Command::new(env!("CARGO_BIN_EXE_txm"))
        .arg(r"\begin{unknown}x\end{unknown}")
        .output()
        .expect("failed to run txm");

    assert!(!output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stderr),
        "error: unknown matrix environment: unknown\n"
    );
}
