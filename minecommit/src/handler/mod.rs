mod chunk_region;
mod entities_region;
mod gzip_nbt;
mod poi_region;
mod raw;

use anyhow::Result;
pub(crate) use chunk_region::ChunkRegionHandler;
pub(crate) use entities_region::EntitiesRegionHandler;
pub(crate) use gzip_nbt::GzipNbtHandler;
pub(crate) use poi_region::PoiRegionHandler;
pub(crate) use raw::RawHandler;

use crate::odb::{OdbReader, OdbWriter};

pub(crate) trait Handler {
    fn flatten(
        self,
        save_dir: &impl OdbReader,
        storage: &mut impl OdbWriter,
    ) -> Result<Vec<String>>;
    fn unflatten(
        self,
        save_dir: &mut impl OdbWriter,
        storage: &impl OdbReader,
    ) -> Result<Vec<String>>;
}

pub(crate) enum CrafterImpl {
    Raw(RawHandler),
    GzipNbt(GzipNbtHandler),
    ChunkRegion(ChunkRegionHandler),
    EntitiesRegion(EntitiesRegionHandler),
    PoiRegion(PoiRegionHandler),
}

impl CrafterImpl {
    pub(crate) fn get_crafters(extra_patterns: Vec<String>) -> Vec<Self> {
        vec![
            Self::ChunkRegion(ChunkRegionHandler {}),
            Self::EntitiesRegion(EntitiesRegionHandler {}),
            Self::PoiRegion(PoiRegionHandler {}),
            Self::Raw(RawHandler { extra_patterns }),
            Self::GzipNbt(GzipNbtHandler {}),
        ]
    }
}

impl Handler for CrafterImpl {
    fn flatten(
        self,
        save_dir: &impl OdbReader,
        storage: &mut impl OdbWriter,
    ) -> Result<Vec<String>> {
        match self {
            Self::Raw(c) => c.flatten(save_dir, storage),
            Self::GzipNbt(c) => c.flatten(save_dir, storage),
            Self::ChunkRegion(c) => c.flatten(save_dir, storage),
            Self::EntitiesRegion(c) => c.flatten(save_dir, storage),
            Self::PoiRegion(c) => c.flatten(save_dir, storage),
        }
    }
    fn unflatten(
        self,
        save_dir: &mut impl OdbWriter,
        storage: &impl OdbReader,
    ) -> Result<Vec<String>> {
        match self {
            Self::Raw(c) => c.unflatten(save_dir, storage),
            Self::GzipNbt(c) => c.unflatten(save_dir, storage),
            Self::ChunkRegion(c) => c.unflatten(save_dir, storage),
            Self::EntitiesRegion(c) => c.unflatten(save_dir, storage),
            Self::PoiRegion(c) => c.unflatten(save_dir, storage),
        }
    }
}
