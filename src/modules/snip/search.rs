use crate::modules::snip::storage::SnippetStore;
use anyhow::Result;
use console::style;

pub fn search_snippets() -> Result<()> {
    let store = SnippetStore::load()?;
    let snippets = store.list();
    
    if snippets.is_empty() {
        println!("No snippets found.");
        return Ok(());
    }

    // List all for now (MVP). Use fuzzy matcher later if requested.
    println!("{}", style("Available Snippets:").bold());
    println!("{:<20} {:<10} {}", "Name", "Usages", "Description");
    println!("{}", "-".repeat(50));
    
    for s in snippets {
        let desc = s.description.as_deref().unwrap_or("");
        println!("{:<20} {:<10} {}", style(&s.name).green(), s.usage_count, desc);
    }

    Ok(())
}
