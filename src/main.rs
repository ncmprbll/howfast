use std::num::ParseIntError;

use clap::Parser;

mod dictionary;
use dictionary::Dictionary;

mod coordinator;
use coordinator::{Coordinator, DebugFlags};

#[derive(Parser, Debug)]
#[command(version, about, long_about =  None)]
struct Args {
    /// Path to the dictionary (see dictionaries for examples)
    #[arg(short = 'f', long, default_value = "dictionaries/english_1k.txt")]
    dictionary_path: String,

    /// The amount of seconds you get for typing
    #[arg(short = 't', long, default_value = "30")]
    typing_time: u64,

    /// Amount of lines to render at a time
    #[arg(short = 'l', long, default_value = "3", value_parser = |x: &str| Ok::<usize, ParseIntError>(x.parse::<usize>()?.max(1)))]
    render_lines: usize,

    /// Enable debugging options with comma separated
    /// values - source_text and text_oveflow are available
    #[arg(short = 'd', long = "debug", verbatim_doc_comment)]
    debug_flags: Option<String>,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let args = Args::parse();

    let mut debug_flags = DebugFlags::NONE;

    if let Some(flags) = args.debug_flags {
        flags.split(",").for_each(|x| match x {
            "source_text" => debug_flags |= DebugFlags::SOURCE_TEXT,
            "text_overflow" => debug_flags |= DebugFlags::TEXT_OVERFLOW,
            _ => (),
        });
    }

    let dictionary = Dictionary::from_file(&args.dictionary_path)?;
    let mut coordinator =
        Coordinator::new(dictionary, args.typing_time, args.render_lines, debug_flags);

    let (wpm, acc, cor, inc, ext, mis) = ratatui::run(|terminal| coordinator.run(terminal))?;

    if args.typing_time == 0 {
        println!("Cool performance - one keystroke per nothing");
        return Ok(());
    }

    println!("WPM: {:.2}", wpm);
    println!("Accuracy: {}%", acc);
    println!(
        "{} correct, {} incorrect, {} extra, {} missed",
        cor, inc, ext, mis
    );

    Ok(())
}
