use std::collections::BTreeSet;
use std::process::ExitCode;

use wordle_solver::types::WORD_LEN;

struct Stats {
    lines: u64,
    kept: u64,
    wrong_case: u64,
    wrong_len: u64,
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [dictionary, output] = args.as_slice() else {
        eprintln!("usage: genlist <dictionary> <output>");
        return ExitCode::from(2);
    };

    let text = match std::fs::read_to_string(dictionary) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("cannot read {dictionary}: {e}");
            return ExitCode::FAILURE;
        }
    };

    let mut words: BTreeSet<&str> = BTreeSet::new();
    let mut stats = Stats {
        lines: 0,
        kept: 0,
        wrong_case: 0,
        wrong_len: 0,
    };

    for raw in text.as_bytes().split(|&b| b == b'\n') {
        let line = match raw.strip_suffix(b"\r") {
            Some(stripped) => stripped,
            None => raw,
        };
        if line.is_empty() {
            continue;
        }
        stats.lines += 1;

        if line.len() != WORD_LEN {
            stats.wrong_len += 1;
            continue;
        }
        let Some(word) = std::str::from_utf8(line).ok() else {
            stats.wrong_case += 1;
            continue;
        };
        if wordle_solver::encode::try_encode(word).is_none() {
            stats.wrong_case += 1;
            continue;
        }

        words.insert(word);
    }

    stats.kept = words.len() as u64;
    if words.is_empty() {
        eprintln!("no {WORD_LEN}-letter words found in {dictionary}");
        return ExitCode::FAILURE;
    }

    let body = words.iter().copied().collect::<Vec<_>>().join("\n");
    if let Err(e) = std::fs::write(output, body + "\n") {
        eprintln!("cannot write {output}: {e}");
        return ExitCode::FAILURE;
    }

    eprintln!(
        "{dictionary}: {} lines, {} usable, {} wrong length, {} non a-z",
        stats.lines, stats.kept, stats.wrong_len, stats.wrong_case
    );
    eprintln!("wrote {output}");
    ExitCode::SUCCESS
}
