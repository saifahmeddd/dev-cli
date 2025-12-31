use crate::modules::snip::storage::{Snippet, SnippetStore};
use anyhow::Result;
use dialoguer::{Input, Select, MultiSelect};
use console::style;

pub fn add_snippet() -> Result<()> {
    let name: String = Input::new()
        .with_prompt("Snippet Name")
        .interact_text()?;

    let content: String = Input::new()
        .with_prompt("Command/Content")
        .interact_text()?;

    let description: String = Input::new()
        .with_prompt("Description (optional)")
        .allow_empty(true)
        .interact_text()?;

    let tags_input: String = Input::new()
        .with_prompt("Tags (comma separated, optional)")
        .allow_empty(true)
        .interact_text()?;

    let tags: Vec<String> = tags_input.split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let snippet = Snippet {
        name: name.clone(),
        content,
        description: if description.is_empty() { None } else { Some(description) },
        tags,
        usage_count: 0,
    };

    let mut store = SnippetStore::load()?;
    store.add(snippet);
    store.save()?;

    println!("{}", style(format!("Snippet '{}' added!", name)).green());
    Ok(())
}
