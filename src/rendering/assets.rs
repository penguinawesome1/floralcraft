use bevy::prelude::*;
use std::collections::HashMap;
use crate::terrain::block::Block;
use crate::config::CONFIG;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerFrameKey {
    Idle,
    Run,
    Missing,
}

impl PlayerFrameKey {
    /// Takes input string and returns its corresponding key.
    /// Used to take config inputs and convert into keys for rendering images.
    pub fn from_string(s: &str) -> Self {
        match s {
            "idle" => Self::Idle,
            "run" => Self::Run,
            _ => Self::Missing,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageKey {
    Player(PlayerFrameKey),
    Block(Block),
}

impl From<Block> for ImageKey {
    fn from(block: Block) -> Self {
        ImageKey::Block(block)
    }
}

impl From<PlayerFrameKey> for ImageKey {
    fn from(player_frame_key: PlayerFrameKey) -> Self {
        ImageKey::Player(player_frame_key)
    }
}

#[derive(Resource)]
pub struct Assets {
    images: HashMap<ImageKey, Handle<Image>>,
    image_paths: HashMap<ImageKey, String>,
}

impl Default for Assets {
    fn default() -> Self {
        Self::new()
    }
}

impl Assets {
    /// Initializes the assets by reading image paths from config.
    pub fn new() -> Self {
        let mut image_paths: HashMap<ImageKey, String> = HashMap::new();

        fn add_image_paths<F, T>(
            image_paths: &mut HashMap<ImageKey, String>,
            map: &HashMap<String, String>,
            converter: F
        )
            where F: Fn(&str) -> T, T: Into<ImageKey>
        {
            for (name_str, path_str) in map {
                let key_variant: T = converter(name_str.as_str());
                image_paths.insert(key_variant.into(), path_str.clone());
            }
        }

        add_image_paths(&mut image_paths, &CONFIG.assets.blocks, |s| Block::from_string(s));
        add_image_paths(&mut image_paths, &CONFIG.assets.player, |s|
            PlayerFrameKey::from_string(s)
        );

        Self {
            images: HashMap::new(),
            image_paths,
        }
    }

    /// Gets an image given its image key.
    /// Ensures the image is loaded and stored if not previously.
    pub fn image(&mut self, asset_server: AssetServer, key: ImageKey) -> Handle<Image> {
        if let Some(handle) = self.images.get(&key) {
            return handle.clone();
        }

        let path_option = self.image_paths.get(&key);

        let loaded_handle = if let Some(path) = path_option {
            asset_server.load(path)
        } else {
            warn!("Image path not found for key: {:?}", key);
            let path: &String = self.image_paths
                .get(&ImageKey::Block(Block::default()))
                .expect("default texture should be loaded");
            asset_server.load(path)
        };

        self.images.insert(key, loaded_handle.clone());
        loaded_handle
    }
}
