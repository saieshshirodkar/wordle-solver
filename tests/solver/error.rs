use std::error::Error as _;

use wordle_solver::error::Error;

fn render(e: &Error) -> String {
    e.to_string()
}

#[test]
fn io_errors_render_the_inner_message() {
    let io = std::io::Error::new(std::io::ErrorKind::NotFound, "missing");
    assert_eq!(render(&Error::Io(io)), "missing");
}

#[test]
fn wordlist_errors_name_the_path_and_word() {
    let e = Error::BadLength {
        path: "d/a.txt".into(),
        word: "abcd".into(),
    };
    assert_eq!(render(&e), "d/a.txt: 'abcd' has wrong length");

    let e = Error::BadChar {
        path: "d/a.txt".into(),
        word: "ab1de".into(),
    };
    assert_eq!(render(&e), "d/a.txt: 'ab1de' has a non a-z character");

    let e = Error::Duplicate {
        path: "d/a.txt".into(),
        word: "soare".into(),
    };
    assert_eq!(render(&e), "d/a.txt: duplicate word 'soare'");

    assert_eq!(
        render(&Error::Empty {
            path: "d/a.txt".into()
        }),
        "d/a.txt: no words"
    );
}

#[test]
fn capacity_errors_report_the_count() {
    assert_eq!(
        render(&Error::TooManyAnswers(70_000)),
        "too many answers (70000)"
    );
    assert_eq!(
        render(&Error::TooManyGuesses(70_000)),
        "guess list too long for u16 indices (70000)"
    );
}

#[test]
fn tree_errors_report_the_node() {
    assert_eq!(
        render(&Error::NoCandidate { node: 42 }),
        "no candidate guess at node 42"
    );
    assert_eq!(
        render(&Error::DepthExceeded { node: 7, depth: 64 }),
        "worst case exceeds 64 guesses at node 7"
    );
}

#[test]
fn input_errors_quote_the_offender() {
    assert_eq!(
        render(&Error::BadSecret(-1)),
        "secret index -1 out of range"
    );
    assert_eq!(
        render(&Error::BadFeedback("xyz".into())),
        "need exactly 5 chars of g/y/b, got 'xyz'"
    );
    assert_eq!(render(&Error::NoMatch), "no word matches that feedback");
}

#[test]
fn only_io_exposes_a_source() {
    let io = std::io::Error::other("boom");
    assert!(Error::Io(io).source().is_some());

    let all = [
        Error::BadLength {
            path: String::new(),
            word: String::new(),
        },
        Error::BadChar {
            path: String::new(),
            word: String::new(),
        },
        Error::Duplicate {
            path: String::new(),
            word: String::new(),
        },
        Error::Empty {
            path: String::new(),
        },
        Error::TooManyAnswers(0),
        Error::TooManyGuesses(0),
        Error::NoCandidate { node: 0 },
        Error::DepthExceeded { node: 0, depth: 0 },
        Error::BadSecret(0),
        Error::BadFeedback(String::new()),
        Error::NoMatch,
    ];
    for e in all {
        assert!(e.source().is_none(), "{e}");
    }
}

#[test]
fn io_converts_via_from() {
    let io = std::io::Error::other("x");
    let e: Error = io.into();
    assert!(matches!(e, Error::Io(_)));
}
