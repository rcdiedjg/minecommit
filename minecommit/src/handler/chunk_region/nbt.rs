use std::collections::HashMap;

use anyhow::{Context, Result};
use simdnbt::borrow;
use simdnbt::owned::{self, BaseNbt, NbtCompound, NbtList, NbtTag};
use simdnbt::{Deserialize, DeserializeError, Serialize};

use super::palette::{
    BlockStateEntry, biome_palette_names, block_palette_entries, dump_biome_data, dump_block_data,
    load_biome, load_block,
};

pub fn split_chunk(nbt: BaseNbt) -> Result<(BaseNbt, SectionsDump)> {
    let name = nbt.name().to_owned();
    let mut nbt = nbt.as_compound();
    let sections_dump = {
        let sections = nbt.remove("sections").ok_or_else(|| {
            anyhow::anyhow!(format!(
                "missing 'sections', all fields: {:#?}",
                nbt.keys().collect::<Vec<_>>()
            ))
        })?;
        let sections = sections
            .list()
            .ok_or(anyhow::anyhow!("expect sections is a NBT list"))?;
        dump_sections(sections)?
    };

    // TODO: extract block/sky light
    if let Some(is_light_on_idx) = nbt.byte_mut("isLightOn") {
        *is_light_on_idx = i8::from(false);
    } else {
        log::trace!("Missing field 'isLightOn', ignored and will be treated as false by game");
    }

    Ok((BaseNbt::new(name, nbt), sections_dump))
}

pub fn restore_chunk(other: BaseNbt, dump: SectionsDump) -> Result<BaseNbt> {
    let name = other.name().to_owned();
    let mut other = other.as_compound();
    other.insert("sections", load_sections(dump)?);
    Ok(BaseNbt::new(name, other))
}

/// Two-pass dump: extract palettes first, then unpack data directly with
/// merged palette indices — no remapping pass needed.
fn dump_sections(sections: &NbtList) -> Result<SectionsDump> {
    let sections_compounds = sections
        .compounds()
        .context("expect sections is a NBT compound list, got: {sections:#?}")?;
    let sections_len = sections_compounds.len();

    // Pass 1 — palette scan only (no data unpacking).
    struct SecMeta {
        y: i8,
        local_biome_palette: Vec<String>,
        local_block_palette: Vec<BlockStateEntry>,
        has_palettes: bool,
    }
    let mut metas: Vec<SecMeta> = Vec::with_capacity(sections_len);
    for (idx, section) in sections_compounds.iter().enumerate() {
        let y = section
            .byte("Y")
            .with_context(|| format!("missing NBT byte 'sections.{idx}.Y', got: {section:#?}"))?;
        if let Some(biome) = section.compound("biomes")
            && let Some(block_states) = section.compound("block_states")
        {
            let local_biome_palette = biome_palette_names(biome)?;
            let local_block_palette = block_palette_entries(block_states)?;
            metas.push(SecMeta {
                y,
                local_biome_palette,
                local_block_palette,
                has_palettes: true,
            });
        } else {
            // push placeholder with empty palettes to keep alignment with sections_compounds
            metas.push(SecMeta {
                y,
                local_biome_palette: Vec::new(),
                local_block_palette: Vec::new(),
                has_palettes: false,
            });
            if idx != 0 && idx != sections_len - 1 {
                anyhow::bail!(
                    "Missing field 'biomes' or/and 'block_states' in 'sections.{idx}' (y={y}), all fields got: {:?}",
                    section.keys().map(|s| s.to_str()).collect::<Vec<_>>()
                );
            } else {
                log::trace!(
                    "Missing field 'biomes' or/and 'block_states' in 'sections.{idx}' (y={y}), all fields got: {:?}",
                    section.keys().map(|s| s.to_str()).collect::<Vec<_>>()
                );
            }
        }
    }

    // Build merged palettes and per-section local→merged index maps.
    let mut biome_merged: Vec<String> = Vec::new();
    let mut biome_hash: HashMap<String, u8> = HashMap::new();
    let mut block_merged: Vec<BlockStateEntry> = Vec::new();
    let mut block_hash: HashMap<BlockStateEntry, u16> = HashMap::new();

    let mut biome_maps: Vec<Vec<u8>> = Vec::with_capacity(metas.len());
    let mut block_maps: Vec<Vec<u16>> = Vec::with_capacity(metas.len());

    for meta in &metas {
        let mut bm = Vec::with_capacity(meta.local_biome_palette.len());
        for name in &meta.local_biome_palette {
            bm.push(*biome_hash.entry(name.clone()).or_insert_with(|| {
                let idx = biome_merged.len() as u8;
                biome_merged.push(name.clone());
                idx
            }));
        }
        biome_maps.push(bm);

        let mut bkm = Vec::with_capacity(meta.local_block_palette.len());
        for entry in &meta.local_block_palette {
            bkm.push(*block_hash.entry(entry.clone()).or_insert_with(|| {
                let idx = block_merged.len() as u16;
                block_merged.push(entry.clone());
                idx
            }));
        }
        block_maps.push(bkm);
    }

    // Pass 2 — unpack data directly with merged-index maps.
    anyhow::ensure!(
        sections_compounds.len() == metas.len(),
        "lengths of sections and metas is not equal: {} != {}",
        sections_compounds.len(),
        metas.len()
    );
    anyhow::ensure!(
        sections_compounds.len() == biome_maps.len(),
        "lengths of sections and biome_maps is not equal: {} != {}",
        sections_compounds.len(),
        biome_maps.len()
    );
    anyhow::ensure!(
        sections_compounds.len() == block_maps.len(),
        "lengths of sections and block_map is not equal: {} != {}",
        sections_compounds.len(),
        block_maps.len()
    );
    let mut sections = Vec::with_capacity(metas.len());
    for (((section, meta), biome_map), block_map) in sections_compounds
        .iter()
        .zip(metas.into_iter())
        .zip(biome_maps.into_iter())
        .zip(block_maps.into_iter())
    {
        if !meta.has_palettes {
            continue;
        }
        let biome = section.compound("biomes").unwrap();
        let block_states = section.compound("block_states").unwrap();
        sections.push(Section {
            y: meta.y,
            biome_data: dump_biome_data(biome, &biome_map)
                .with_context(|| format!("failed to dump biome data for y={}", meta.y))?,
            block_data: dump_block_data(block_states, &block_map)
                .with_context(|| format!("failed to dump block data for y={}", meta.y))?,
        });
    }

    Ok(SectionsDump {
        biome_palette: biome_merged,
        block_palette: block_merged,
        sections,
    })
}

/// Reconstruct per-section NBT from the chunk-level shared palettes.
/// Each section extracts only the subset of palette entries it actually
/// uses, so the reconstructed NBT is as compact as the original.
fn load_sections(dump: SectionsDump) -> Result<NbtList> {
    let list = dump
        .sections
        .into_iter()
        .map(|section| {
            // Find which merged indices this section actually uses
            let mut used_biomes: Vec<bool> = vec![false; dump.biome_palette.len()];
            for &v in section.biome_data.iter() {
                used_biomes[v as usize] = true;
            }
            let mut used_blocks: Vec<bool> = vec![false; dump.block_palette.len()];
            for &v in section.block_data.iter() {
                used_blocks[v as usize] = true;
            }

            // Build local sub-palettes and remap tables
            let mut biome_sub: Vec<String> =
                Vec::with_capacity(used_biomes.iter().filter(|&&x| x).count());
            let mut biome_remap: Vec<u8> = vec![0; dump.biome_palette.len()];
            for (i, used) in used_biomes.iter().enumerate() {
                if *used {
                    biome_remap[i] = biome_sub.len() as u8;
                    biome_sub.push(dump.biome_palette[i].clone());
                }
            }

            let mut block_sub: Vec<BlockStateEntry> =
                Vec::with_capacity(used_blocks.iter().filter(|&&x| x).count());
            let mut block_remap: Vec<u16> = vec![0; dump.block_palette.len()];
            for (i, used) in used_blocks.iter().enumerate() {
                if *used {
                    block_remap[i] = block_sub.len() as u16;
                    block_sub.push(dump.block_palette[i].clone());
                }
            }

            // Remap section data to local sub-palette indices
            let mut biome_data = section.biome_data;
            for v in biome_data.iter_mut() {
                *v = biome_remap[*v as usize];
            }
            let mut block_data = section.block_data;
            for v in block_data.iter_mut() {
                *v = block_remap[*v as usize];
            }

            let kvs = vec![
                ("Y".into(), owned::NbtTag::Byte(section.y)),
                (
                    "biomes".into(),
                    owned::NbtTag::Compound(load_biome(biome_sub, biome_data)?),
                ),
                (
                    "block_states".into(),
                    owned::NbtTag::Compound(load_block(block_sub, block_data)?),
                ),
            ];
            Ok(NbtCompound::from_values(kvs))
        })
        .collect::<Result<Vec<_>>>()?;
    Ok(NbtList::from(list))
}

/// A single section within the dump — palette data is stored per-chunk in
/// `SectionsDump`, so `Section` only carries palette *indices*.
struct Section {
    y: i8,
    biome_data: Box<[u8; 64]>,
    block_data: Box<[u16; 4096]>,
}

impl Serialize for Section {
    fn to_compound(self) -> owned::NbtCompound {
        let mut nbt = owned::NbtCompound::new();
        if let Some(tag) = simdnbt::ToNbtTag::to_optional_nbt_tag(self.y) {
            nbt.insert("y", tag);
        }
        nbt.insert(
            "biome_data",
            owned::NbtTag::ByteArray(self.biome_data.to_vec()),
        );
        nbt.insert(
            "block_data",
            owned::NbtTag::List(NbtList::from(
                self.block_data
                    .to_vec()
                    .into_iter()
                    .map(|x| x as i16)
                    .collect::<Vec<_>>(),
            )),
        );
        nbt
    }
}

impl Deserialize for Section {
    fn from_compound(nbt: borrow::NbtCompound) -> Result<Self, DeserializeError> {
        Ok(Self {
            y: simdnbt::FromNbtTag::from_optional_nbt_tag(nbt.get("y"))?
                .ok_or(DeserializeError::MismatchedFieldType("Section::y".into()))?,

            biome_data: {
                let arr =
                    nbt.byte_array("biome_data")
                        .ok_or(DeserializeError::MismatchedFieldType(
                            "Section::biome_data".into(),
                        ))?;
                let vec = arr.to_owned();
                Box::<[u8; 64]>::try_from(vec.into_boxed_slice()).map_err(|_| {
                    DeserializeError::MismatchedFieldType("Section::biome_data (bad length)".into())
                })?
            },

            block_data: {
                let list = nbt
                    .list("block_data")
                    .ok_or(DeserializeError::MismatchedFieldType(
                        "Section::block_data".into(),
                    ))?;
                let shorts = list.shorts().ok_or(DeserializeError::MismatchedFieldType(
                    "Section::block_data".into(),
                ))?;
                let vec: Vec<u16> = shorts.iter().map(|&x| x as u16).collect();
                Box::<[u16; 4096]>::try_from(vec.into_boxed_slice()).map_err(|_| {
                    DeserializeError::MismatchedFieldType("Section::block_data (bad length)".into())
                })?
            },
        })
    }
}

fn block_palette_to_nbt_tags(entries: Vec<BlockStateEntry>) -> Vec<owned::NbtTag> {
    entries
        .into_iter()
        .map(|entry| {
            let kvs: Vec<(simdnbt::Mutf8String, NbtTag)> = if entry.properties.is_empty() {
                vec![(
                    "name".into(),
                    NbtTag::String(simdnbt::Mutf8String::from_string(entry.name)),
                )]
            } else {
                let props_kvs: Vec<(simdnbt::Mutf8String, NbtTag)> = entry
                    .properties
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            simdnbt::Mutf8String::from_string(k),
                            NbtTag::String(simdnbt::Mutf8String::from_string(v)),
                        )
                    })
                    .collect();
                vec![
                    (
                        "name".into(),
                        NbtTag::String(simdnbt::Mutf8String::from_string(entry.name)),
                    ),
                    (
                        "properties".into(),
                        NbtTag::Compound(NbtCompound::from_values(props_kvs)),
                    ),
                ]
            };
            owned::NbtTag::Compound(NbtCompound::from_values(kvs))
        })
        .collect()
}

fn block_palette_from_nbt_tags(
    list: borrow::NbtList,
) -> Result<Vec<BlockStateEntry>, DeserializeError> {
    let compounds = list
        .compounds()
        .ok_or(DeserializeError::MismatchedFieldType(
            "block_palette (not compound list)".into(),
        ))?;
    compounds
        .into_iter()
        .map(|c| {
            let name = c
                .string("name")
                .ok_or(DeserializeError::MismatchedFieldType(
                    "block_palette entry name".into(),
                ))?
                .to_string();
            let properties = if let Some(props) = c.compound("properties") {
                props
                    .iter()
                    .map(|(k, v)| {
                        let v = v.string().ok_or(DeserializeError::MismatchedFieldType(
                            "block_palette properties value".into(),
                        ))?;
                        Ok((k.to_string(), v.to_string()))
                    })
                    .collect::<Result<Vec<_>, DeserializeError>>()?
            } else {
                Vec::new()
            };
            Ok(BlockStateEntry { name, properties })
        })
        .collect()
}

/// Serialized dump format for a chunk's sections.
/// Palettes are stored once per chunk, sections carry only index data.
pub struct SectionsDump {
    biome_palette: Vec<String>,
    block_palette: Vec<BlockStateEntry>,
    sections: Vec<Section>,
}

impl Serialize for SectionsDump {
    fn to_compound(self) -> owned::NbtCompound {
        let mut nbt = owned::NbtCompound::new();
        nbt.insert(
            "biome_palette",
            NbtList::from(
                self.biome_palette
                    .into_iter()
                    .map(|s| simdnbt::Mutf8String::from_string(s))
                    .collect::<Vec<_>>(),
            ),
        );
        nbt.insert(
            "block_palette",
            NbtList::from(block_palette_to_nbt_tags(self.block_palette)),
        );
        // Reuse Section's Serialize for the list
        nbt.insert(
            "sections",
            NbtList::from(
                self.sections
                    .into_iter()
                    .map(|s| NbtTag::Compound(s.to_compound()))
                    .collect::<Vec<_>>(),
            ),
        );
        nbt
    }
}

impl Deserialize for SectionsDump {
    fn from_compound(nbt: borrow::NbtCompound) -> Result<Self, DeserializeError> {
        Ok(Self {
            biome_palette: {
                let list =
                    nbt.list("biome_palette")
                        .ok_or(DeserializeError::MismatchedFieldType(
                            "SectionsDump::biome_palette".into(),
                        ))?;
                let strings = list.strings().ok_or(DeserializeError::MismatchedFieldType(
                    "SectionsDump::biome_palette".into(),
                ))?;
                strings.iter().map(|s| s.to_string()).collect()
            },
            block_palette: {
                let list =
                    nbt.list("block_palette")
                        .ok_or(DeserializeError::MismatchedFieldType(
                            "SectionsDump::block_palette".into(),
                        ))?;
                block_palette_from_nbt_tags(list)?
            },
            sections: {
                let list = nbt
                    .list("sections")
                    .ok_or(DeserializeError::MismatchedFieldType(
                        "SectionsDump::sections".into(),
                    ))?;
                let compounds = list
                    .compounds()
                    .ok_or(DeserializeError::MismatchedFieldType(
                        "SectionsDump::sections".into(),
                    ))?;
                compounds
                    .into_iter()
                    .map(|c| Section::from_compound(c))
                    .collect::<Result<Vec<_>, _>>()?
            },
        })
    }
}
