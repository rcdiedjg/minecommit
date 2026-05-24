use std::collections::HashMap;

use anyhow::Result;

type Biome = String;
type BlockState = (String, Box<[(String, String)]>);

pub struct MinecraftDataMapping {
    biomes: Vec<Biome>,
    biome_mapping: HashMap<Biome, u8>,
    block_states: Vec<BlockState>,
    block_state_mapping: HashMap<BlockState, u16>,
}

impl MinecraftDataMapping {
    pub fn register_biome(&mut self, biome: &str) {
        if !self.biome_mapping.contains_key(biome) {
            self.biomes.push(biome.to_owned());
            self.biome_mapping
                .insert(biome.to_owned(), self.biome_mapping.len() as u8);
        }
    }
    pub fn biome_id_from_name(&self, name: &str) -> Result<u8> {
        todo!()
    }
    pub fn biome_name_from_id(&self, id: u8) -> Result<String> {
        todo!()
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
    ) -> Result<u16> {
        todo!()
    }
    pub fn block_name_and_props_from_state_id(
        &self,
        state_id: u16,
    ) -> Result<(String, Vec<(String, String)>)> {
        todo!()
    }
}
