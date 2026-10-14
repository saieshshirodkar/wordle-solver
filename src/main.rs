mod cli;

use std::io::Write;
use std::time::Instant;

use wordle_solver::builder::{Builder, threads_available};
use wordle_solver::cache::{self, Meta};
use wordle_solver::encode::decode;
use wordle_solver::error::{Error, Result};
use wordle_solver::fmt::{push_fixed, scaled};
use wordle_solver::solver::{self, Secrets};
use wordle_solver::tree::Tree;
use wordle_solver::types::Word;
use wordle_solver::wordlist::load_words;

use cli::{Opt, USAGE, answer_feedback, interactive, report};

fn main() {
    if let Err(e) = run() {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let opt = parse_opt();
    if opt.help {
        print!("{USAGE}");
        return Ok(());
    }

    let answers = load_words(&opt.answers)?;
    let probes = if opt.allowed.is_empty() {
        Vec::new()
    } else {
        load_words(&opt.allowed)?
    };

    let mut guesses = probes.clone();
    guesses.extend_from_slice(&answers);
    if guesses.len() > u16::MAX as usize {
        return Err(Error::TooManyGuesses(guesses.len()));
    }

    let hash = cache::hash_lists(&probes, &answers);

    let out = std::io::stdout();
    let mut w = out.lock();

    let (tree, from_cache) = resolve_tree(&opt, &answers, &probes, &mut w)?;

    writeln!(
        w,
        "answers:   {}\nprobes:    {}\ncandidates: {}\nfirst:     {}",
        answers.len(),
        probes.len(),
        guesses.len(),
        decode(guesses[tree.guess_at(0) as usize])
    )?;

    let save = !opt.no_cache && !from_cache;
    if opt.benchmark || save {
        let counts = solver::secrets(&answers);
        let s = Secrets::new(&answers, &counts);
        let t0 = Instant::now();
        let b = solver::run_bench(&tree, &guesses, &s, threads_available());
        if opt.benchmark {
            report(&b, answers.len(), t0.elapsed().as_secs_f64(), &mut w)?;
        }
        if save {
            let mut meta = Meta::new(probes.len(), answers.len(), b.worst, b.total);
            meta.list_hash = hash;
            let result = cache::save(&opt.cache, &tree, &meta);
            if opt.benchmark {
                match result {
                    Ok(()) => writeln!(w, "  cached to {}", opt.cache)?,
                    Err(e) => writeln!(w, "  cache write failed: {e}")?,
                }
            }
        }
    }

    if !opt.feedback.is_empty() {
        return answer_feedback(&opt.feedback, &tree, &guesses, &mut w);
    }

    interactive(&tree, &guesses, &mut w)
}

fn parse_opt() -> Opt {
    cli::parse_args()
}

fn resolve_tree<W: Write>(
    opt: &Opt,
    answers: &[Word],
    probes: &[Word],
    w: &mut W,
) -> Result<(Tree, bool)> {
    let t0 = Instant::now();

    if !opt.no_cache {
        let hash = cache::hash_lists(probes, answers);
        if let Some((tree, _)) = cache::load(&opt.cache, hash, probes.len(), answers.len())? {
            let mut line = format!("tree:      cached, {} nodes (", tree.node_count());
            push_fixed::<1>(&mut line, scaled(t0.elapsed().as_secs_f64(), 1e3));
            line.push_str(" ms)");
            writeln!(w, "{line}")?;
            return Ok((tree, true));
        }
    }

    let tree = build_optimized(answers, probes, opt.rounds)?;
    let mut line = format!("tree:      built, {} nodes (", tree.node_count());
    push_fixed::<0>(&mut line, scaled(t0.elapsed().as_secs_f64(), 1e3));
    line.push_str(" ms)");
    writeln!(w, "{line}")?;
    Ok((tree, false))
}

fn build_optimized(answers: &[Word], probes: &[Word], rounds: usize) -> Result<Tree> {
    let mut b = Builder::new(answers.to_vec(), probes.to_vec())?;
    b.build()?;
    let history = b.optimize(rounds)?;
    if std::env::var("WORDLE_VERBOSE").is_ok_and(|v| v != "0") {
        let mut line = String::from("refine:");
        for v in &history {
            line.push(' ');
            push_fixed::<4>(&mut line, scaled(*v, 1e4));
        }
        eprintln!("{line}");
    }
    Ok(b.into_parts().0)
}
