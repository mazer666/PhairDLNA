//! Device matrix and observed trial records acceptance tests (INT-102, F-07 AC-07).
//!
//! What: verifies that every recorded device trial in `qa/device-matrix.toml` specifies
//!       mandatory fields: device model, firmware version, client application, media format,
//!       and an observed outcome.
//! Why: INT-102 requires recording every device-test result in the interoperability matrix.

use std::path::Path;

#[test]
fn device_matrix_records_have_mandatory_attributes() {
    let matrix_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("qa/device-matrix.toml"))
        .expect("resolve device-matrix.toml path");

    assert!(
        matrix_path.exists(),
        "qa/device-matrix.toml must exist (INT-102, F-07)"
    );

    let content = std::fs::read_to_string(&matrix_path).expect("read device-matrix.toml");
    let toml: toml::Value = toml::from_str(&content).expect("parse TOML device matrix");

    let tests = toml
        .get("tests")
        .and_then(|t| t.as_array())
        .expect("tests array in device matrix");

    assert!(
        !tests.is_empty(),
        "device matrix must have at least one observed trial (INT-102)"
    );

    for (idx, trial) in tests.iter().enumerate() {
        assert!(
            trial.get("device").is_some(),
            "trial {} must name device model (INT-102)",
            idx
        );
        assert!(
            trial.get("firmware").is_some(),
            "trial {} must name firmware version (INT-102)",
            idx
        );
        assert!(
            trial.get("client").is_some(),
            "trial {} must name client application (INT-102)",
            idx
        );
        assert!(
            trial.get("media_format").is_some(),
            "trial {} must name media format (INT-102)",
            idx
        );
        assert!(
            trial.get("outcome").is_some(),
            "trial {} must record observed outcome (INT-102)",
            idx
        );
    }
}
