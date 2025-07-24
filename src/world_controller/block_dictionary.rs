use block_dictionary::{Block, load_blocks};
use once_cell::sync::Lazy;

static BLOCK_DICTIONARY: Lazy<Result<Vec<Block>, block_dictionary::CliError>> =
    Lazy::new(|| load_blocks("Blocks.toml"));

pub fn definition(value: u8) -> &'static Block {
    match BLOCK_DICTIONARY.as_ref() {
        Ok(dictionary) => dictionary.get(value as usize).unwrap_or(&Block::MISSING),
        Err(e) => {
            eprintln!("Error loading block dictionary: {}", e);
            &Block::MISSING
        }
    }
}
