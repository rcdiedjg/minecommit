use anyhow::{Context, Result};
use simdnbt::owned::{BaseNbt, NbtCompound, NbtTag};
use std::io::Cursor;

use super::Handler;
use crate::odb::{OdbReader, OdbWriter};
use crate::utils::nbt::{load_nbt, sort_nbt};
use crate::utils::region::{parse_xz, read_region, write_region};

const FLATTEN_PATTERNS: &[&str] = &["**/entities/r.*.*.mca"];

const UNFLATTEN_PATTERNS: &[&str] = &["**/entities/r.*.*.mca/timestamp-header"];

pub(crate) struct EntitiesRegionHandler;

impl Handler for EntitiesRegionHandler {
    fn workspace(&self) -> &'static str {
        "entities-region"
    }

    fn flatten(self, save: &impl OdbReader, storage: &mut impl OdbWriter) -> Result<Vec<String>> {
        let mut processed = Vec::new();
        for pattern in FLATTEN_PATTERNS {
            for key in save.glob(pattern)? {
                log::info!("Process entities region file {key}");
                let data = save.get(&key)?;
                let filename = key.split('/').next_back().unwrap_or("");
                let (region_x, region_z) = parse_xz(filename)
                    .with_context(|| format!("failed to parse region coordinates from {key}"))?;
                let Some((timestamp_header, chunks)) =
                    read_region(Cursor::new(data), region_x, region_z)
                        .with_context(|| format!("failed to read region from {key}"))?
                else {
                    processed.push(key);
                    continue;
                };
                storage.put(&format!("{key}/timestamp-header"), &timestamp_header)?;

                // Parse and sort all chunk NBTs
                let mut result: Vec<(i32, i32, BaseNbt)> = chunks
                    .into_iter()
                    .map(|(chunk_x, chunk_z, raw_bytes)| {
                        let raw_nbt = load_nbt(Cursor::new(&raw_bytes))
                            .context("failed to load chunk nbt")?;
                        let sorted_nbt = sort_nbt(raw_nbt);
                        Ok((chunk_x, chunk_z, sorted_nbt))
                    })
                    .collect::<Result<Vec<_>>>()?;

                // Sort by (cz, cx) for deterministic ordering
                result
                    .sort_unstable_by(|(cx1, cz1, ..), (cx2, cz2, ..)| (cz1, cx1).cmp(&(cz2, cx2)));

                // Build and write entities.nbt (all chunk NBTs in one compound)
                {
                    let mut entities_compound = NbtCompound::new();
                    for (chunk_x, chunk_z, nbt) in &mut result {
                        let key_str = format!("c.{}.{}", chunk_x, chunk_z);
                        entities_compound.insert(
                            key_str,
                            NbtTag::Compound(
                                std::mem::replace(nbt, BaseNbt::default()).as_compound(),
                            ),
                        );
                    }
                    let entities_nbt = simdnbt::owned::BaseNbt::new("", entities_compound);
                    let mut entities_buf = Vec::new();
                    entities_nbt.write(&mut entities_buf);
                    storage.put(&format!("{key}/entities.nbt"), &entities_buf)?;
                }

                processed.push(key);
            }
        }
        Ok(processed)
    }

    fn unflatten(self, save: &mut impl OdbWriter, storage: &impl OdbReader) -> Result<Vec<String>> {
        let mut processed = Vec::new();
        for pattern in UNFLATTEN_PATTERNS {
            for ts_key in storage.glob(pattern)? {
                log::info!("Process entities region file (timestamp header) {ts_key}");
                let Some(region_key) = ts_key.strip_suffix("/timestamp-header") else {
                    continue;
                };
                let filename = region_key.split('/').next_back().unwrap_or("");
                let (region_x, region_z) = parse_xz(filename)
                    .with_context(|| format!("failed to parse region coordinates from {ts_key}"))?;
                let timestamp_header = storage.get(&ts_key)?;

                // Read entities.nbt (all chunk NBTs in one compound)
                let entities_key = format!("{region_key}/entities.nbt");
                let entities_data = storage
                    .get(&entities_key)
                    .with_context(|| format!("failed to read {entities_key}"))?;
                let entities_nbt = load_nbt(std::io::Cursor::new(&entities_data))
                    .context("failed to load entities nbt")?;
                let mut entities_compound = entities_nbt.as_compound();

                // Extract coordinates from compound keys
                let mut coords: Vec<(i32, i32)> = entities_compound
                    .keys()
                    .filter_map(|key| {
                        let s = key.to_str();
                        s.strip_prefix("c.").and_then(|rest| {
                            let (x_str, z_str) = rest.split_once('.')?;
                            let x = x_str.parse::<i32>().ok()?;
                            let z = z_str.parse::<i32>().ok()?;
                            Some((x, z))
                        })
                    })
                    .collect();
                coords.sort_unstable_by(|(x1, z1), (x2, z2)| (z1, x1).cmp(&(z2, x2)));

                // Reconstruct chunks from compound
                let mut chunks = Vec::new();
                for (chunk_x, chunk_z) in coords {
                    let key_str = format!("c.{}.{}", chunk_x, chunk_z);
                    let nbt_tag = entities_compound
                        .remove(&key_str)
                        .ok_or_else(|| anyhow::anyhow!("missing '{}' in entities nbt", key_str))?;
                    let nbt_compound = nbt_tag
                        .into_compound()
                        .ok_or_else(|| anyhow::anyhow!("expect '{}' is NBT Compound", key_str))?;
                    let mut buf = Vec::new();
                    simdnbt::owned::BaseNbt::new("", nbt_compound).write(&mut buf);
                    chunks.push((chunk_x, chunk_z, buf));
                }

                let mut mca_buf = Vec::with_capacity(200 * 1024); // 200KiB
                write_region(
                    region_x,
                    region_z,
                    &timestamp_header[..4096]
                        .try_into()
                        .context("timestamp header must be at least 4096 bytes")?,
                    chunks,
                    Cursor::new(&mut mca_buf),
                )
                .with_context(|| format!("failed to write region for {ts_key}"))?;
                save.put(region_key, &mca_buf)?;

                processed.push(ts_key);
                processed.push(entities_key);
            }
        }
        Ok(processed)
    }
}
