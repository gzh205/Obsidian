use crate::core::wad::tree::utils::find_parent_item_mut;
use std::{
    io::{Read, Seek},
    iter::Peekable,
    sync::Arc,
};

use indexmap::IndexMap;
use thiserror::Error;
use tracing::info;
use uuid::Uuid;

mod item;
mod utils;

pub use item::*;

use crate::state::WadHashtable;

use self::utils::{
    add_item_to_parent, find_parent_item, sort_parent_items, traverse_parent_items,
    traverse_parent_items_mut,
};

use super::{Wad, WadChunk};

#[derive(Error, Debug)]
pub enum WadTreeError {
    #[error("invalid item name (chunk_path: {chunk_path:#0x})")]
    InvalidItemName { chunk_path: u64 },

    #[error("failed to create item (item_path: {item_path})")]
    ItemCreationFailure { item_path: String },

    #[error("existing file: (file_path: {file_path})")]
    ExistingFile { file_path: String },
}

#[derive(Debug)]
pub struct WadTree {
    wad_id: Uuid,
    wad_path: Arc<str>,
    items: IndexMap<WadTreeItemKey, WadTreeItem>,
    selected_items: Vec<WadTreeItemKey>,
}

impl WadTree {
    pub fn from_wad<TSource>(
        wad: &Wad<TSource>,
        wad_id: Uuid,
        wad_path: impl Into<Arc<str>>,
        hashtable: &WadHashtable,
    ) -> Result<WadTree, WadTreeError>
    where
        TSource: Read + Seek,
    {
        info!("creating wad tree for wad (wad_id: {})", wad_id);

        let mut tree = WadTree {
            wad_id,
            wad_path: wad_path.into(),
            items: IndexMap::default(),
            selected_items: vec![],
        };

        for (_, chunk) in wad.chunks() {
            let path = Self::resolve_chunk_path(chunk.path_hash, &hashtable);
            let mut path_components = path.split('/').peekable();

            add_item_to_parent(&mut tree, &mut path_components, chunk, &hashtable)?;
        }

        sort_parent_items(&mut tree);

        Ok(tree)
    }

    fn resolve_chunk_path(path_hash: u64, hashtable: &WadHashtable) -> Arc<str> {
        match hashtable.items().get(&path_hash) {
            Some(path) => path.clone(),
            None => format!("{:#0x}", path_hash).into(),
        }
    }

    pub fn wad_id(&self) -> Uuid {
        self.wad_id
    }
    pub fn wad_path(&self) -> &str {
        &self.wad_path
    }

    pub fn set_selected_items(&mut self, selected_items: impl IntoIterator<Item = WadTreeItemKey>) {
        self.selected_items = Vec::from_iter(selected_items);
    }
}

impl WadTreeParent for WadTree {
    fn is_root(&self) -> bool {
        true
    }

    fn items(&self) -> &IndexMap<WadTreeItemKey, WadTreeItem> {
        &self.items
    }

    fn items_mut(&mut self) -> &mut IndexMap<WadTreeItemKey, WadTreeItem> {
        &mut self.items
    }

    fn traverse_items(&self, mut cb: &mut impl FnMut(&WadTreeItem)) {
        traverse_parent_items(self, &mut cb)
    }
    fn traverse_items_mut(&mut self, mut cb: &mut impl FnMut(&mut WadTreeItem)) {
        traverse_parent_items_mut(self, &mut cb)
    }

    fn find_item(&self, condition: impl Fn(&WadTreeItem) -> bool) -> Option<&WadTreeItem> {
        find_parent_item(self, &condition)
    }
    fn find_item_mut(
        &mut self,
        condition: impl Fn(&WadTreeItem) -> bool,
    ) -> Option<&mut WadTreeItem> {
        find_parent_item_mut(self, &condition)
    }
}

impl WadTreeParentInternal for WadTree {
    fn add_item(
        &mut self,
        path_components: &mut Peekable<std::str::Split<char>>,
        chunk: &WadChunk,
        hashtable: &WadHashtable,
    ) -> Result<(), WadTreeError> {
        add_item_to_parent(self, path_components, chunk, &hashtable)
    }
}

impl WadTreePathable for WadTree {
    fn id(&self) -> Uuid {
        uuid::uuid!("00000000-0000-0000-0000-000000000000")
    }

    fn name(&self) -> Arc<str> {
        "".into()
    }
    fn path(&self) -> Arc<str> {
        "".into()
    }
    fn name_hash(&self) -> u64 {
        0
    }
    fn path_hash(&self) -> u64 {
        0
    }
}
