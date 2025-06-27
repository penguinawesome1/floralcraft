use macroquad::prelude::{ Texture2D, load_texture };
use std::collections::HashMap;
use crate::terrain::block::Block;
use crate::config::CONFIG;
use crate::game::player::PlayerFrameKey;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageKey {
    Player(PlayerFrameKey),
    Block(Block),
}

pub struct Assets {
    images: HashMap<ImageKey, Texture2D>,
    image_paths: HashMap<ImageKey, String>,
}

impl Assets {
    /// Initializes the assets by reading image paths from config.
    pub async fn new() -> Result<Self, String> {
        let mut image_paths: HashMap<ImageKey, String> = HashMap::new();

        // match block string to key
        for (name_str, path_str) in &CONFIG.assets.blocks {
            let block_name: Block = Block::from_string(name_str.as_str());
            image_paths.insert(ImageKey::Block(block_name), path_str.clone());
        }

        // match player string to key
        for (frame_str, path_str) in &CONFIG.assets.player {
            let player_frame_key: PlayerFrameKey = PlayerFrameKey::from_string(frame_str.as_str());
            image_paths.insert(ImageKey::Player(player_frame_key), path_str.clone());
        }

        Ok(Self {
            images: HashMap::new(),
            image_paths,
        })
    }

    /// Gets the loaded image that matches the key.
    /// Loads images into the hash map the first time they are called.
    pub async fn get_or_load_image(&mut self, key: ImageKey) -> Result<&Texture2D, String> {
        if !self.images.contains_key(&key) {
            let path = self.image_paths
                .get(&key)
                .ok_or_else(|| format!("Image path not found in config for {:?}", key))?;

            if path.is_empty() {
                return Err(format!("Attempted to load image with empty path for {:?}", key));
            }

            let texture: Texture2D = load_texture(path).await.map_err(|e|
                format!("Failed to load texture from '{}' for {:?}: {:?}", path, key, e)
            )?;

            self.images.insert(key, texture);
        }

        Ok(self.images.get(&key).unwrap())
    }
}
