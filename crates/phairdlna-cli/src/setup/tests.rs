//! Tests for the first-run CLI setup flow.
//!
//! What: cover prompt recovery, generated configuration and plain-language guidance.
//! Why: setup edits a user's configuration and must reject incomplete or malformed answers.
//! Example: a truncated conversation returns an error instead of writing a partial file.

use super::*;

/// The answers of an owner who named the server and one folder.
fn answers() -> Answers {
    Answers {
        device_name: "Living room".to_owned(),
        roots: vec![("Movies".to_owned(), "/Users/owner/Movies".to_owned())],
        start_at_login: true,
    }
}

/// The positive case: the questions are asked and the answers are read.
#[test]
fn the_questions_are_asked_and_the_answers_are_read() {
    let input = "Living room\n/Users/owner/Movies\n\nyes\n";
    let mut reader = std::io::BufReader::new(input.as_bytes());
    let mut prompts: Vec<u8> = Vec::new();

    let answers = ask(&mut reader, &mut prompts).expect("the answers are read");

    assert_eq!(answers.device_name, "Living room");
    assert_eq!(
        answers.roots,
        vec![("Movies".to_owned(), "/Users/owner/Movies".to_owned())]
    );
    assert!(answers.start_at_login);
    let text = String::from_utf8(prompts).expect("the prompts are text");
    assert!(text.contains("What should the server be called"), "{text}");
}

/// The positive case: the configuration the helper writes is a valid one.
#[test]
fn the_configuration_the_helper_writes_is_valid() {
    let text = render(&answers());
    assert!(text.contains("device_name = \"Living room\""));
    assert!(text.contains("name = \"Movies\""));
    assert!(check(&text).is_ok(), "{text}");
}

/// The negative case: an answer that stops in the middle is refused.
#[test]
fn an_answer_that_stops_in_the_middle_is_refused() {
    let input = "Living room\n";
    let mut reader = std::io::BufReader::new(input.as_bytes());
    let mut prompts: Vec<u8> = Vec::new();
    assert!(ask(&mut reader, &mut prompts).is_err());
}

/// The negative case: a name with a quote does not break the file.
#[test]
fn a_name_with_a_quote_does_not_break_the_file() {
    let answers = Answers {
        device_name: "The \"Living\" room".to_owned(),
        roots: Vec::new(),
        start_at_login: false,
    };
    let text = render(&answers);
    if let Err(err) = check(&text) {
        panic!("check failed: {err}\nText:\n{text}");
    }
}

/// The negative case: the explanation answers the three questions.
#[test]
fn the_explanation_answers_the_three_questions() {
    let text = explanation();
    assert!(text.contains("PhairDLNA app's local connection"), "{text}");
    assert!(text.contains("Only programs on"), "{text}");
    assert!(text.contains("admin_enabled = false"), "{text}");
    assert!(!text.contains("administration page"), "{text}");
}
