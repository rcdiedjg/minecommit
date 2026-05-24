use std::collections::HashMap;

use simdnbt::owned::{BaseNbt, NbtCompound, NbtList, NbtTag};

type Biome = String;
type BlockState = (String, Box<[(String, String)]>);

pub struct MinecraftDataMapping {
    biomes: Vec<Biome>,
    biome_mapping: HashMap<Biome, u8>,
    block_states: Vec<BlockState>,
    block_state_mapping: HashMap<BlockState, u16>,
}

impl Default for MinecraftDataMapping {
    fn default() -> Self {
        Self {
            biomes: Vec::new(),
            biome_mapping: HashMap::new(),
            block_states: Vec::new(),
            block_state_mapping: HashMap::new(),
        }
    }
}

impl MinecraftDataMapping {
    pub fn register_biome(&mut self, biome: &str) {
        if !self.biome_mapping.contains_key(biome) {
            self.biomes.push(biome.to_owned());
            self.biome_mapping
                .insert(biome.to_owned(), self.biome_mapping.len() as u8);
        }
    }
    pub fn biome_id_from_name(&self, name: &str) -> Option<u8> {
        self.biome_mapping.get(name).cloned()
    }
    pub fn biome_name_from_id(&self, id: u8) -> Option<Biome> {
        self.biomes.get(id as usize).cloned()
    }
    pub fn register_block_state(&mut self, name: &str, props: &[(&str, &str)]) {
        let key = (
            name.to_string(),
            props
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        if !self.block_state_mapping.contains_key(&key) {
            self.block_states.push(key.to_owned());
            self.block_state_mapping
                .insert(key.to_owned(), self.block_state_mapping.len() as u16);
        }
    }
    pub fn block_state_id_from_name_and_props(
        &self,
        name: &str,
        props: &[(&str, &str)],
    ) -> Option<u16> {
        let key = (
            name.to_string(),
            props
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        );
        self.block_state_mapping.get(&key).cloned()
    }
    pub fn block_name_and_props_from_state_id(&self, state_id: u16) -> Option<BlockState> {
        self.block_states.get(state_id as usize).cloned()
    }
    pub fn to_nbt(self) -> (BaseNbt, BaseNbt) {
        let mut biomes_compound = NbtCompound::new();
        biomes_compound.insert("biomes", self.biomes);

        let mut block_states_compound = NbtCompound::new();
        let block_state_tags: Vec<NbtTag> = self
            .block_states
            .into_iter()
            .map(|(name, props)| {
                let mut bs = NbtCompound::new();
                bs.insert("name", name);
                let mut props_compound = NbtCompound::new();
                for (k, v) in props.iter() {
                    props_compound.insert(k.as_str(), v.as_str());
                }
                bs.insert("props", props_compound);
                NbtTag::Compound(bs)
            })
            .collect();
        block_states_compound.insert("block_states", NbtList::from(block_state_tags));

        (BaseNbt::from(biomes_compound), BaseNbt::from(block_states_compound))
    }
    pub fn from_nbt(biomes_nbt: BaseNbt, block_states_nbt: BaseNbt) -> Self {
        let biomes: Vec<Biome> = biomes_nbt
            .list("biomes")
            .and_then(|l| l.strings())
            .map(|ss| ss.iter().map(|s| s.to_string()).collect())
            .unwrap_or_default();

        let mut biome_mapping = HashMap::with_capacity(biomes.len());
        for (i, b) in biomes.iter().enumerate() {
            biome_mapping.insert(b.clone(), i as u8);
        }

        let block_states: Vec<BlockState> = block_states_nbt
            .list("block_states")
            .and_then(|l| l.compounds())
            .map(|cs| {
                cs.iter()
                    .map(|c| {
                        let name = c.string("name").map(|s| s.to_string()).unwrap_or_default();
                        let props: Box<[(String, String)]> = c
                            .compound("props")
                            .map(|pc| {
                                pc.iter()
                                    .map(|(k, v)| {
                                        let val = match v {
                                            NbtTag::String(s) => s.to_string(),
                                            _ => String::new(),
                                        };
                                        (k.to_string(), val)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        (name, props)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut block_state_mapping = HashMap::with_capacity(block_states.len());
        for (i, bs) in block_states.iter().enumerate() {
            block_state_mapping.insert(bs.clone(), i as u16);
        }

        Self {
            biomes,
            biome_mapping,
            block_states,
            block_state_mapping,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        let mut mapping = MinecraftDataMapping::default();
        mapping.register_biome("minecraft:plains");
        mapping.register_biome("minecraft:desert");
        mapping.register_biome("minecraft:forest");
        mapping.register_block_state("minecraft:stone", &[]);
        mapping.register_block_state("minecraft:oak_log", &[("axis", "y")]);
        mapping.register_block_state(
            "minecraft:redstone_wire",
            &[
                ("east", "side"),
                ("north", "none"),
                ("power", "0"),
                ("south", "side"),
                ("west", "none"),
            ],
        );

        let (biomes_nbt, block_states_nbt) = mapping.to_nbt();
        let restored = MinecraftDataMapping::from_nbt(biomes_nbt, block_states_nbt);

        assert_eq!(restored.biome_id_from_name("minecraft:plains"), Some(0));
        assert_eq!(restored.biome_id_from_name("minecraft:desert"), Some(1));
        assert_eq!(restored.biome_id_from_name("minecraft:forest"), Some(2));
        assert_eq!(
            restored.biome_name_from_id(0),
            Some("minecraft:plains".into())
        );
        assert_eq!(
            restored.biome_name_from_id(1),
            Some("minecraft:desert".into())
        );
        assert_eq!(
            restored.biome_name_from_id(2),
            Some("minecraft:forest".into())
        );
        assert_eq!(restored.biome_id_from_name("nonexistent"), None);
        assert_eq!(restored.biome_name_from_id(3), None);

        assert_eq!(
            restored.block_state_id_from_name_and_props("minecraft:stone", &[]),
            Some(0)
        );
        assert_eq!(
            restored.block_state_id_from_name_and_props("minecraft:oak_log", &[("axis", "y")]),
            Some(1)
        );
        assert_eq!(
            restored.block_state_id_from_name_and_props(
                "minecraft:redstone_wire",
                &[
                    ("east", "side"),
                    ("north", "none"),
                    ("power", "0"),
                    ("south", "side"),
                    ("west", "none")
                ],
            ),
            Some(2)
        );

        let (name, props) = restored.block_name_and_props_from_state_id(1).unwrap();
        assert_eq!(name, "minecraft:oak_log");
        assert_eq!(props.as_ref(), &[("axis".to_string(), "y".to_string())]);
    }

    #[test]
    fn empty_mapping() {
        let mapping = MinecraftDataMapping::default();
        let (biomes_nbt, block_states_nbt) = mapping.to_nbt();
        let restored = MinecraftDataMapping::from_nbt(biomes_nbt, block_states_nbt);

        assert_eq!(restored.biome_id_from_name("minecraft:plains"), None);
        assert_eq!(restored.biome_name_from_id(0), None);
        assert_eq!(
            restored.block_state_id_from_name_and_props("minecraft:stone", &[]),
            None
        );
        assert_eq!(restored.block_name_and_props_from_state_id(0), None);
    }
}
