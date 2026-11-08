use crate::schematic::SpongeSchematic;
use fastnbt::{ByteArray, IntArray};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap};
use std::io::{self, Read};

#[derive(Deserialize, Debug)]
pub struct StructurePaletteEntry {
    #[serde(rename = "Name")]
    pub name: String,
    #[serde(rename = "Properties")]
    pub properties: Option<BTreeMap<String, String>>,
}

impl StructurePaletteEntry {
    #[allow(dead_code)]
    pub fn to_block_state_string(&self) -> String {
        self.to_block_state_string_with_speed(1.0)
    }

    pub fn to_block_state_string_with_speed(&self, speed: f64) -> String {
        match &self.properties {
            Some(props) if !props.is_empty() => {
                let mut modified = props.clone();
                if (speed - 1.0).abs() > f64::EPSILON && self.name.contains("repeater") {
                    if let Some(d_str) = modified.get_mut("delay") {
                        if let Ok(d) = d_str.parse::<f64>() {
                            let new_d = ((d / speed).round() as i32).clamp(1, 4);
                            *d_str = new_d.to_string();
                        }
                    }
                }
                let props_str = modified
                    .iter()
                    .map(|(k, v)| format!("{}={}", k, v))
                    .collect::<Vec<_>>()
                    .join(",");
                format!("{}[{}]", self.name, props_str)
            }
            _ => self.name.clone(),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct StructureBlock {
    pub pos: Vec<i32>,
    pub state: i32,
}

#[derive(Deserialize, Debug)]
pub struct VanillaStructure {
    #[serde(rename = "DataVersion")]
    pub data_version: Option<i32>,
    pub size: Vec<i32>,
    pub palette: Option<Vec<StructurePaletteEntry>>,
    pub palettes: Option<Vec<Vec<StructurePaletteEntry>>>,
    pub blocks: Vec<StructureBlock>,
}

fn write_varint(mut val: u32, buf: &mut Vec<u8>) {
    while val >= 0x80 {
        buf.push((val as u8 & 0x7F) | 0x80);
        val >>= 7;
    }
    buf.push(val as u8);
}

pub fn convert_structure_to_schematic(data: &[u8], speed: f64) -> io::Result<SpongeSchematic> {
    // Try gunzip first
    let raw_nbt = if data.starts_with(&[0x1F, 0x8B]) {
        let mut decoder = GzDecoder::new(data);
        let mut buf = Vec::new();
        decoder.read_to_end(&mut buf)?;
        buf
    } else {
        data.to_vec()
    };

    let structure: VanillaStructure = fastnbt::from_bytes(&raw_nbt)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("Structure NBT error: {}", e)))?;

    let palette_entries = if let Some(p) = structure.palette {
        p
    } else if let Some(mut ps) = structure.palettes {
        if ps.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Empty palettes in structure NBT",
            ));
        }
        ps.remove(0)
    } else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No palette found in structure NBT",
        ));
    };

    let state_strings: Vec<String> = palette_entries
        .into_iter()
        .map(|entry| entry.to_block_state_string_with_speed(speed))
        .collect();

    // Map coordinates (x, y, z) -> state string and compute real bounding box
    let mut min_x = 0i32;
    let mut min_y = 0i32;
    let mut min_z = 0i32;
    let mut max_x = structure.size.first().copied().unwrap_or(1) - 1;
    let mut max_y = structure.size.get(1).copied().unwrap_or(1) - 1;
    let mut max_z = structure.size.get(2).copied().unwrap_or(1) - 1;

    let mut block_grid: HashMap<(i32, i32, i32), &str> = HashMap::new();
    for b in &structure.blocks {
        if b.pos.len() == 3 {
            let x = b.pos[0];
            let y = b.pos[1];
            let z = b.pos[2];
            min_x = min_x.min(x);
            max_x = max_x.max(x);
            min_y = min_y.min(y);
            max_y = max_y.max(y);
            min_z = min_z.min(z);
            max_z = max_z.max(z);
            if let Some(state_str) = state_strings.get(b.state as usize) {
                block_grid.insert((x, y, z), state_str.as_str());
            }
        }
    }

    let width = (max_x - min_x + 1) as i16;
    let height = (max_y - min_y + 1) as i16;
    let length = (max_z - min_z + 1) as i16;

    // Build Sponge Palette
    let air_str = "minecraft:air";
    let mut schem_palette: HashMap<String, i32> = HashMap::new();
    schem_palette.insert(air_str.into(), 0);
    let mut next_id = 1i32;

    for &state in block_grid.values() {
        if !schem_palette.contains_key(state) {
            schem_palette.insert(state.to_string(), next_id);
            next_id += 1;
        }
    }

    let palette_max = schem_palette.len() as i32;

    // Build BlockData (y * Length + z) * Width + x
    let mut block_data_bytes = Vec::new();
    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let state = block_grid.get(&(x, y, z)).copied().unwrap_or(air_str);
                let id = *schem_palette.get(state).unwrap_or(&0) as u32;
                write_varint(id, &mut block_data_bytes);
            }
        }
    }

    let block_data_i8: Vec<i8> = block_data_bytes.into_iter().map(|b| b as i8).collect();

    Ok(SpongeSchematic {
        version: 2,
        data_version: structure.data_version.unwrap_or(3953).max(3953),
        width,
        height,
        length,
        offset: IntArray::new(vec![min_x, min_y, min_z]),
        palette_max,
        palette: schem_palette,
        block_data: ByteArray::new(block_data_i8),
    })
}
