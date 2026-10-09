mod dictionary;
use dictionary::Dictionary;

mod coordinator;
use coordinator::{Coordinator, DebugFlags};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;

    let dictionary = Dictionary::from_file("dictionaries/english_1k.txt")?;
    let mut coordinator = Coordinator::new(dictionary);
    coordinator.debug_flags = DebugFlags(DebugFlags::SOURCE_TEXT);

    let (wpm, acc, cor, inc, ext, mis) = ratatui::run(|terminal| coordinator.run(terminal))?;

    println!("WPM: {:.2}", wpm);
    println!("Accuracy: {}%", acc);
    println!(
        "{} correct, {} incorrect, {} extra, {} missed",
        cor, inc, ext, mis
    );

    Ok(())
}
