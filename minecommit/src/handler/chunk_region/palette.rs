use anyhow::{Context, Result};
use simdnbt::{
    Mutf8String,
    owned::{NbtCompound, NbtList, NbtTag},
};

type Cube<T, const SIZE: usize> = [[[T; SIZE]; SIZE]; SIZE];

/// A block state entry in a local section palette.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BlockStateEntry {
    pub name: String,
    pub properties: Vec<(String, String)>,
}

#[inline]
fn fast_count_bits(n: usize) -> usize {
    let m = n.saturating_sub(1);
    usize::BITS as usize - m.leading_zeros() as usize
}

// ---------------------------------------------------------------------------
// Generic palette ↔ cube conversions
// ---------------------------------------------------------------------------

/// Unpack Minecraft packed long-array palette data into a dense 3-D cube.
/// `palette_entries` maps a local palette index to the value stored in the
/// cube — for disjoint-index dumps this is the merged-index table, for
/// self-contained dumps it is `&[0u8, 1u8, 2u8, …]`.
fn unpack_palette<T: Copy + Default + Eq, const SIZE: usize>(
    packed_rows: &[u64],
    palette_entries: &[T],
    count_bits: impl Fn(usize) -> usize,
) -> Box<Cube<T, SIZE>> {
    let bits = count_bits(palette_entries.len());
    let chunk_size = 64 / bits;
    let mask = (1u64 << bits) - 1;
    let mut cube = Box::new([[[T::default(); SIZE]; SIZE]; SIZE]);

    let flattened_cube = cube.as_flattened_mut().as_flattened_mut();
    for (cube_chunk, row) in flattened_cube.chunks_mut(chunk_size).zip(packed_rows) {
        let mut row = *row;
        for cube_elem in cube_chunk.iter_mut() {
            let palette_index = (row & mask) as usize;
            *cube_elem = palette_entries[palette_index];
            row >>= bits;
        }
    }

    cube
}

/// Pack a dense 3-D cube back into Minecraft long-array rows, using the
/// known palette size to determine the encoding bit width.
fn pack_cube<T: Into<u64> + Copy, const SIZE: usize>(
    cube: &Cube<T, SIZE>,
    palette_len: usize,
    count_bits: impl Fn(usize) -> usize,
) -> Vec<u64> {
    let bits = count_bits(palette_len);
    if bits == 0 {
        return vec![];
    }
    let chunk_size = 64 / bits;
    let mut rows = vec![0u64; (SIZE * SIZE * SIZE).div_ceil(chunk_size)];
    let flattened = cube.as_flattened().as_flattened();
    for (cube_chunk, row) in flattened.chunks(chunk_size).zip(rows.iter_mut()) {
        *row = 0;
        for &elem in cube_chunk.iter().rev() {
            *row <<= bits;
            *row |= elem.into();
        }
    }
    rows
}

// ---------------------------------------------------------------------------
// Biome helpers
// ---------------------------------------------------------------------------

/// Extract only the palette string list from a biome NBT compound
/// (no data unpacking).
pub fn biome_palette_names(nbt: &NbtCompound) -> Result<Vec<String>> {
    Ok(nbt
        .list("palette")
        .with_context(|| format!("missing NBT list 'palette' in biome, got: {nbt:#?}"))?
        .strings()
        .with_context(|| "expect biome 'palette' is a NBT string list")?
        .iter()
        .map(|s| s.to_string())
        .collect())
}

/// Unpack biome data from NBT into a 64-byte flat array.  Each byte is
/// obtained by translating the local palette index through `local_to_merged`
/// so callers can produce merged-indexed data in one shot.
pub fn dump_biome_data(nbt: &NbtCompound, local_to_merged: &[u8]) -> Result<Box<[u8; 64]>> {
    let cube: Box<Cube<u8, 4>> = if let Some(rows) = nbt.long_array("data") {
        unpack_palette::<u8, 4>(bytemuck::cast_slice(rows), local_to_merged, fast_count_bits)
    } else {
        // No data → homogeneous single-value palette
        let val = local_to_merged.first().copied().unwrap_or(0);
        bytemuck::allocation::cast_box(Box::new([val; 64]))
    };
    Ok(bytemuck::allocation::cast_box(cube))
}

/// Reconstruct a biome NBT compound from a local palette and 64-byte
/// indexed data.
pub fn load_biome(palette: Vec<String>, data: Box<[u8; 64]>) -> Result<NbtCompound> {
    let cube: Box<Cube<u8, 4>> = bytemuck::allocation::cast_box(data);

    let first = cube[0][0][0];
    let is_homo = cube.iter().flatten().flatten().all(|&x| x == first);

    let kvs = if is_homo {
        let entry = palette.get(first as usize).ok_or_else(|| {
            anyhow::anyhow!(
                "biome palette index {first} out of bounds (len={})",
                palette.len()
            )
        })?;
        vec![(
            "palette".into(),
            NbtTag::List(NbtList::from(vec![Mutf8String::from_string(entry.clone())])),
        )]
    } else {
        let rows = pack_cube(&cube, palette.len(), fast_count_bits);
        let entries: Vec<Mutf8String> = palette
            .into_iter()
            .map(|s| Mutf8String::from_string(s))
            .collect();
        vec![
            ("data".into(), NbtTag::LongArray(bytemuck::cast_vec(rows))),
            ("palette".into(), NbtTag::List(NbtList::from(entries))),
        ]
    };
    Ok(NbtCompound::from_values(kvs))
}

// ---------------------------------------------------------------------------
// Block-state helpers
// ---------------------------------------------------------------------------

/// Extract only the palette entry list from a block-state NBT compound
/// (no data unpacking).
pub fn block_palette_entries(nbt: &NbtCompound) -> Result<Vec<BlockStateEntry>> {
    nbt.list("palette")
        .with_context(|| format!("missing NBT list 'palette' in block_states, got: {nbt:#?}"))?
        .compounds()
        .with_context(|| "expect block 'palette' is a NBT compound list")?
        .into_iter()
        .enumerate()
        .map(|(idx, entry)| {
            let name = entry
                .string("Name")
                .with_context(|| {
                    format!("missing NBT string 'palette.{idx}.Name', got: {entry:#?}")
                })?
                .to_string();
            let properties = if let Some(props) = entry.compound("Properties") {
                props
                    .iter()
                    .map(|(k, value)| {
                        let v = value.string().with_context(|| {
                            format!(
                                "expect 'palette.{idx}.Properties.{}' is a NBT string",
                                k.to_str()
                            )
                        })?;
                        Ok((k.to_string(), v.to_string()))
                    })
                    .collect::<Result<Vec<_>>>()?
            } else {
                Vec::new()
            };
            Ok(BlockStateEntry { name, properties })
        })
        .collect::<Result<Vec<_>>>()
}

/// Unpack block-state data from NBT into a 4096-u16 flat array.  Each u16
/// is obtained by translating the local palette index through
/// `local_to_merged`.
pub fn dump_block_data(nbt: &NbtCompound, local_to_merged: &[u16]) -> Result<Box<[u16; 4096]>> {
    let count_bits = |n: usize| fast_count_bits(n).max(4);
    let cube: Box<Cube<u16, 16>> = if let Some(rows) = nbt.long_array("data") {
        unpack_palette::<u16, 16>(bytemuck::cast_slice(rows), local_to_merged, count_bits)
    } else {
        let val = local_to_merged.first().copied().unwrap_or(0);
        bytemuck::allocation::cast_box(Box::new([val; 4096]))
    };
    Ok(bytemuck::allocation::cast_box(cube))
}

/// Reconstruct a block-state NBT compound from a local palette and
/// 4096-u16 indexed data.
pub fn load_block(palette: Vec<BlockStateEntry>, data: Box<[u16; 4096]>) -> Result<NbtCompound> {
    let cube: Box<Cube<u16, 16>> = bytemuck::allocation::cast_box(data);
    let count_bits = |n: usize| fast_count_bits(n).max(4);
    let bits = count_bits(palette.len());
    let rows = pack_cube(&cube, palette.len(), count_bits);

    let entries: Vec<NbtTag> = palette
        .into_iter()
        .map(|entry| {
            let kvs: Vec<(Mutf8String, NbtTag)> = if entry.properties.is_empty() {
                vec![(
                    "Name".into(),
                    NbtTag::String(Mutf8String::from_string(entry.name)),
                )]
            } else {
                let props_kvs: Vec<(Mutf8String, NbtTag)> = entry
                    .properties
                    .into_iter()
                    .map(|(k, v)| {
                        (
                            Mutf8String::from_string(k),
                            NbtTag::String(Mutf8String::from_string(v)),
                        )
                    })
                    .collect();
                vec![
                    (
                        "Name".into(),
                        NbtTag::String(Mutf8String::from_string(entry.name)),
                    ),
                    (
                        "Properties".into(),
                        NbtTag::Compound(NbtCompound::from_values(props_kvs)),
                    ),
                ]
            };
            NbtTag::Compound(NbtCompound::from_values(kvs))
        })
        .collect();

    let kvs = if rows.is_empty() || bits == 0 {
        vec![("palette".into(), NbtTag::List(NbtList::from(entries)))]
    } else {
        vec![
            ("data".into(), NbtTag::LongArray(bytemuck::cast_vec(rows))),
            ("palette".into(), NbtTag::List(NbtList::from(entries))),
        ]
    };
    Ok(NbtCompound::from_values(kvs))
}
