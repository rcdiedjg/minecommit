use anyhow::{Context, Result};
use flate2::Compression;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use simdnbt::owned::{BaseNbt, NbtCompound, NbtList};
use std::io::{Cursor, Read, Write};

use super::Handler;
use crate::{
    odb::{OdbReader, OdbWriter},
    utils::nbt::{dump_nbt, load_nbt, sort_nbt},
};

const GZIP_NBT_GLOB_PATTERNS: &[&str] = &["**/*.dat", "**/*.dat_old", "**/*.nbt"];

pub(crate) struct GzipNbtHandler {}

impl Handler for GzipNbtHandler {
    fn workspace(&self) -> &'static str {
        "gzip-nbt"
    }

    fn flatten(self, save: &impl OdbReader, storage: &mut impl OdbWriter) -> Result<Vec<String>> {
        let mut processed = Vec::new();
        for pattern in GZIP_NBT_GLOB_PATTERNS {
            for key in save.glob(pattern)? {
                log::info!("Process gzip nbt file {key}");
                let compressed = save.get(&key)?;
                let mut decoder = GzDecoder::new(compressed.as_slice());
                let decompressed = if decoder.header().is_some() {
                    let mut decompressed = Vec::new();
                    decoder
                        .read_to_end(&mut decompressed)
                        .context("failed to decompress gzip data")?;
                    decompressed
                } else {
                    log::info!(
                        "Failed to decompress because header is invalid, treat as uncompressed"
                    );
                    compressed
                };
                let sorted = {
                    let nbt = load_nbt(Cursor::new(&decompressed))?;
                    let nbt = sort_nbt(nbt);

                    // Sort recipe book for player data
                    let nbt = {
                        let match_file_old = glob::Pattern::new("playerdata/*.dat")
                            .context("failed to compile glob pattern")?
                            .matches(&key);
                        let match_file_new = glob::Pattern::new("players/data/*.dat")
                            .context("failed to compile glob pattern")?
                            .matches(&key);
                        if match_file_old || match_file_new {
                            let name = nbt.name().to_owned();
                            let mut comp = nbt.as_compound();
                            sort_recipe_book(&mut comp);
                            BaseNbt::new(name, comp)
                        } else {
                            nbt
                        }
                    };

                    // Sort recipe book for player data in level.dat
                    let nbt = {
                        if glob::Pattern::new("level.dat")
                            .context("failed to compile glob pattern")?
                            .matches(&key)
                        {
                            let name = nbt.name().to_owned();
                            let mut comp = nbt.as_compound();
                            if let Some(data) = comp.compound_mut("Data")
                                && let Some(player) = data.compound_mut("Player")
                            {
                                sort_recipe_book(player);
                            }
                            BaseNbt::new(name, comp)
                        } else {
                            nbt
                        }
                    };

                    // Sort player attributes in level.dat
                    let nbt = {
                        if key == "level.dat" || key == "level.dat_old" {
                            let name = nbt.name().to_owned();
                            let mut comp = nbt.as_compound();
                            if let Some(data) = comp.compound_mut("Data")
                                && let Some(player) = data.compound_mut("Player")
                            {
                                sort_player_attributes(player);
                            }
                            BaseNbt::new(name, comp)
                        } else {
                            nbt
                        }
                    };

                    dump_nbt(nbt, decompressed.len())?
                };
                storage.put(&key, &sorted)?;

                processed.push(key);
            }
        }
        Ok(processed)
    }

    fn unflatten(self, save: &mut impl OdbWriter, storage: &impl OdbReader) -> Result<Vec<String>> {
        let mut processed = Vec::new();
        for pattern in GZIP_NBT_GLOB_PATTERNS {
            for key in storage.glob(pattern)? {
                log::info!("Process gzip nbt file {key}");
                let data = storage.get(&key)?;
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder
                    .write_all(&data)
                    .context("failed to write data to gzip encoder")?;
                let compressed = encoder
                    .finish()
                    .context("failed to finish gzip compression")?;
                save.put(&key, &compressed)?;

                processed.push(key);
            }
        }
        Ok(processed)
    }
}

fn sort_recipe_book(comp: &mut NbtCompound) {
    if let Some(recipe_book) = comp.compound_mut("recipeBook") {
        for key in ["recipes", "toBeDisplayed"] {
            if let Some(NbtList::String(strings)) = recipe_book.list_mut(key) {
                strings.sort_unstable_by(|a, b| a.as_bytes().cmp(b.as_bytes()));
            }
        }
    }
}

fn sort_player_attributes(comp: &mut NbtCompound) {
    if let Some(NbtList::Compound(attributes)) = comp.list_mut("attributes") {
        attributes.sort_by(|a, b| {
            a.string("id")
                .map(|s| s.as_bytes())
                .cmp(&b.string("id").map(|s| s.as_bytes()))
        });
    }
}
