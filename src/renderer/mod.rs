use bevy::prelude::*;
use std::collections::HashMap;
use ahash::AHasher;
use std::hash::BuildHasherDefault;
use std::collections::hash_map::Entry;

#[derive(Default, Resource)]
pub struct Assets {
    images: HashMap<String, Handle<Image>, BuildHasherDefault<AHasher>>,
}

impl Assets {
    pub fn image(&mut self, asset_server: AssetServer, path: &str) -> Handle<Image> {
        match self.images.entry(path.to_string()) {
            Entry::Occupied(entry) => { entry.get().clone() }
            Entry::Vacant(entry) => {
                let handle: Handle<Image> = asset_server.load(path);
                entry.insert(handle.clone());
                handle
            }
        }
    }
}
