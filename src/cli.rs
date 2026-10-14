use std::io::{BufRead, Write};

use wordle_solver::error::{Error, Result};
use wordle_solver::fmt::{fixed, push_fixed, scaled};
use wordle_solver::pattern::parse_feedback;
use wordle_solver::solver::Bench;
use wordle_solver::tree::{NONE, Tree};
use wordle_solver::types::ALL_GREEN;

pub(crate) const USAGE: &str = "\
usage: wordle-rs [--answers PATH] [--allowed PATH] [--cache PATH]
                 [--no-cache] [--no-bench] [--rounds N] [FEEDBACK]
  FEEDBACK  5 chars of g/y/b to skip the benchmark and print the next guess
";

#[derive(Debug)]
pub(crate) struct Opt {
    pub answers: String,
    pub allowed: String,
    pub cache: String,
    pub benchmark: bool,
    pub no_cache: bool,
    pub help: bool,
    pub rounds: usize,
    pub feedback: String,
}

impl Default for Opt {
    fn default() -> Self {
        Self {
            answers: "data/answers.txt".into(),
            allowed: "data/guesses.txt".into(),
            cache: "wordle-tree.bin".into(),
            benchmark: true,
            no_cache: false,
            help: false,
            rounds: 6,
            feedback: String::new(),
        }
    }
}

pub(crate) fn parse_args() -> Opt {
    let mut o = Opt::default();
    let mut it = std::env::args().skip(1);
    while let Some(a) = it.next() {
        match a.as_str() {
            "--no-bench" => o.benchmark = false,
            "--rounds" => {
                let raw = it.next().unwrap_or_default();
                o.rounds = if let Ok(v) = raw.parse::<usize>() {
                    v
                } else {
                    eprintln!("error: --rounds wants a number, got '{raw}'");
                    eprint!("{USAGE}");
                    std::process::exit(2);
                };
            }
            "--no-cache" => o.no_cache = true,
            "--cache" => o.cache = it.next().unwrap_or_default(),
            "--answers" => o.answers = it.next().unwrap_or_default(),
            "--allowed" => o.allowed = it.next().unwrap_or_default(),
            "-h" | "--help" => o.help = true,
            s if !s.starts_with('-') && o.feedback.is_empty() => o.feedback.push_str(s),
            _ => {}
        }
    }
    if !o.feedback.is_empty() {
        o.benchmark = false;
    }
    o
}

pub(crate) fn report<W: Write>(b: &Bench, n: usize, secs: f64, w: &mut W) -> Result<()> {
    let mut hist = [0u32; 17];
    for &c in &b.counts {
        if let Ok(i) = usize::try_from(c)
            && i < hist.len()
        {
            hist[i] += 1;
        }
    }

    write!(w, "\ndistribution:")?;
    for (i, &h) in hist.iter().enumerate().skip(1) {
        if h > 0 {
            write!(w, " {i}:{h}")?;
        }
    }
    writeln!(w)?;

    let avg4 = if b.failed == 0 {
        let d = i64::try_from(n).unwrap_or(1).max(1);
        (b.total * 10_000 + d / 2) / d
    } else {
        scaled(b.avg, 1e4)
    };

    let mut solve = String::from("  solve all: ");
    push_fixed::<3>(&mut solve, scaled(secs, 1e6));
    solve.push_str(" ms");

    writeln!(
        w,
        "\nresults\n  average:  {}\n  stddev:   {}\n  worst:    {}\n  sum:      {}{solve}",
        fixed::<4>(avg4),
        fixed::<4>(scaled(b.stddev, 1e4)),
        b.worst,
        b.total
    )?;

    if b.failed > 0 {
        writeln!(w, "  unsolved: {} (walk failed)", b.failed)?;
    }
    Ok(())
}

pub(crate) fn answer_feedback<W: Write>(
    fb: &str,
    tree: &Tree,
    guesses: &[u64],
    w: &mut W,
) -> Result<()> {
    let Some(pat) = parse_feedback(fb) else {
        return Err(Error::BadFeedback(fb.to_owned()));
    };
    let next = tree.lookup(0, pat);
    if next == NONE {
        return Err(Error::NoMatch);
    }
    writeln!(
        w,
        "{}",
        wordle_solver::encode::decode(guesses[tree.guess_at(next) as usize])
    )?;
    Ok(())
}

pub(crate) fn interactive<W: Write>(tree: &Tree, guesses: &[u64], w: &mut W) -> Result<()> {
    writeln!(
        w,
        "\n=== interactive ===\nfeedback: 5 chars, g=green y=yellow b=gray (q quits)"
    )?;

    let mut node: u16 = 0;
    let mut line = String::new();
    let mut input = std::io::stdin().lock();

    loop {
        if tree.terminal(node) {
            writeln!(
                w,
                "\n  guess: {}  <- answer",
                wordle_solver::encode::decode(guesses[tree.guess_at(node) as usize])
            )?;
            break;
        }

        write!(
            w,
            "  guess: {}\n  feedback: ",
            wordle_solver::encode::decode(guesses[tree.guess_at(node) as usize])
        )?;
        w.flush()?;

        line.clear();
        if input.read_line(&mut line)? == 0 {
            break;
        }
        let t = line.trim();
        if t.is_empty() {
            continue;
        }
        if matches!(t, "q" | "Q" | "quit") {
            break;
        }

        let Some(pat) = parse_feedback(t) else {
            writeln!(w, "  need exactly 5 chars of g/y/b")?;
            continue;
        };
        if pat == ALL_GREEN {
            writeln!(w, "  solved")?;
            break;
        }

        let next = tree.lookup(node, pat);
        if next == NONE {
            writeln!(w, "  no word matches that feedback")?;
            break;
        }
        node = next;
    }
    Ok(())
}
