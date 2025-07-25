use block_dictionary::{Block, load_blocks};
use std::sync::LazyLock;

static BLOCK_DICTIONARY: LazyLock<Result<Vec<Block>, block_dictionary::CliError>> =
    LazyLock::new(|| load_blocks("Blocks.toml"));

pub fn definition(value: u8) -> &'static Block {
    match BLOCK_DICTIONARY.as_ref() {
        Ok(dictionary) => dictionary.get(value as usize).unwrap_or(&Block::MISSING),
        Err(e) => {
            eprintln!("Error loading block dictionary: {}", e);
            &Block::MISSING
        }
    }
}
