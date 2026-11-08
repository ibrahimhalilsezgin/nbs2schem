use crate::nbs::NbsSong;
use fastnbt::{ByteArray, IntArray};
use flate2::write::GzEncoder;
use flate2::Compression;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

#[derive(Serialize)]
#[serde(rename = "Schematic")]
pub struct SpongeSchematic {
    #[serde(rename = "Version")]
    pub version: i32,
    #[serde(rename = "DataVersion")]
    pub data_version: i32,
    #[serde(rename = "Width")]
    pub width: i16,
    #[serde(rename = "Height")]
    pub height: i16,
    #[serde(rename = "Length")]
    pub length: i16,
    #[serde(rename = "Offset")]
    pub offset: IntArray,
    #[serde(rename = "PaletteMax")]
    pub palette_max: i32,
    #[serde(rename = "Palette")]
    pub palette: HashMap<String, i32>,
    #[serde(rename = "BlockData")]
    pub block_data: ByteArray,
}

pub fn get_instrument_info(instrument_id: u8) -> (&'static str, &'static str) {
    match instrument_id {
        0 => ("harp", "minecraft:oak_planks"),
        1 => ("basedrum", "minecraft:stone"),
        2 => ("snare", "minecraft:sand"),
        3 => ("hat", "minecraft:glass"),
        4 => ("bass", "minecraft:oak_wood"),
        5 => ("flute", "minecraft:clay"),
        6 => ("bell", "minecraft:gold_block"),
        7 => ("guitar", "minecraft:white_wool"),
        8 => ("chime", "minecraft:packed_ice"),
        9 => ("xylophone", "minecraft:bone_block"),
        10 => ("iron_xylophone", "minecraft:iron_block"),
        11 => ("cow_bell", "minecraft:soul_sand"),
        12 => ("didgeridoo", "minecraft:pumpkin"),
        13 => ("bit", "minecraft:emerald_block"),
        14 => ("banjo", "minecraft:hay_block"),
        15 => ("pling", "minecraft:glowstone"),
        _ => ("harp", "minecraft:oak_planks"),
    }
}

fn write_varint(mut val: u32, buf: &mut Vec<u8>) {
    while val >= 0x80 {
        buf.push((val as u8 & 0x7F) | 0x80);
        val >>= 7;
    }
    buf.push(val as u8);
}

pub struct BuildOptions {
    pub data_version: i32,
    pub tempo_scale: f64,
}

impl Default for BuildOptions {
    fn default() -> Self {
        Self {
            data_version: 3953, // MC 1.21+ / 26.3
            tempo_scale: 1.0,
        }
    }
}

pub fn build_schematic(song: &NbsSong, options: &BuildOptions) -> io::Result<SpongeSchematic> {
    let mut block_map: HashMap<(i32, i32, i32), String> = HashMap::new();

    // Group notes by target redstone tick
    let base_tps = if song.tempo > 0 {
        (song.tempo as f64 / 100.0) * options.tempo_scale
    } else {
        10.0 * options.tempo_scale
    };

    let mut tick_groups: BTreeMap<u32, Vec<(u8, u8)>> = BTreeMap::new();
    for note in &song.notes {
        let sec = (note.tick as f64) / base_tps;
        let rs_tick = (sec * 10.0).round() as u32;
        tick_groups
            .entry(rs_tick)
            .or_default()
            .push((note.instrument, note.key));
    }

    // Start block: Push button on smooth stone block facing south
    // Y=0: smooth stone base, Y=1: solid block with button
    block_map.insert((0, 0, 0), "minecraft:smooth_stone".into());
    block_map.insert((0, 1, 0), "minecraft:smooth_stone".into());
    block_map.insert(
        (0, 1, -1),
        "minecraft:oak_button[face=wall,facing=north,powered=false]".into(),
    );

    let mut current_z = 1i32;
    let mut prev_tick = 0u32;
    let mut station_index = 0usize;

    for (&tick, notes) in &tick_groups {
        let delta = if station_index == 0 {
            1
        } else {
            (tick - prev_tick).max(1)
        };

        // Decompose delta into repeaters
        let mut rem = delta;
        let mut repeater_delays = Vec::new();
        while rem > 4 {
            repeater_delays.push(4);
            rem -= 4;
        }
        if rem > 0 {
            repeater_delays.push(rem);
        }

        // Place repeaters along spine (X=0)
        for rep_delay in repeater_delays {
            block_map.insert((0, 0, current_z), "minecraft:smooth_stone".into());
            block_map.insert(
                (0, 1, current_z),
                format!(
                    "minecraft:repeater[delay={},facing=south,locked=false,powered=false]",
                    rep_delay
                ),
            );
            current_z += 1;
        }

        // Station node: Strongly powered solid block
        block_map.insert((0, 0, current_z), "minecraft:smooth_stone".into());
        block_map.insert((0, 1, current_z), "minecraft:smooth_stone".into());

        // Deduplicate notes for this tick
        let mut unique_notes = notes.clone();
        unique_notes.sort_unstable();
        unique_notes.dedup();

        let side: i32 = if station_index % 2 == 0 { 1 } else { -1 };

        // Distribute notes along side rib
        for (j, &(inst, key)) in unique_notes.iter().enumerate() {
            let wire_k = (j / 2) as i32 + 1;
            let z_offset = if j % 2 == 0 { -1 } else { 1 };
            let wire_x = side * wire_k;
            let note_x = wire_x;
            let note_z = current_z + z_offset;

            // Wire block
            block_map.insert((wire_x, 0, current_z), "minecraft:smooth_stone".into());
            block_map.insert((wire_x, 1, current_z), "minecraft:redstone_wire".into());

            // Note block and instrument base
            let (inst_name, under_block) = get_instrument_info(inst);
            let pitch = (key as i32 - 33).clamp(0, 24) as u8;

            block_map.insert((note_x, 0, note_z), under_block.into());
            block_map.insert(
                (note_x, 1, note_z),
                format!(
                    "minecraft:note_block[instrument={},note={},powered=false]",
                    inst_name, pitch
                ),
            );
            // Ensure air above note block
            block_map.insert((note_x, 2, note_z), "minecraft:air".into());
        }

        current_z += 1;
        prev_tick = tick;
        station_index += 1;
    }

    if block_map.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "No notes found in song to build schematic",
        ));
    }

    // Determine bounding box
    let mut min_x = i32::MAX;
    let mut max_x = i32::MIN;
    let mut min_y = i32::MAX;
    let mut max_y = i32::MIN;
    let mut min_z = i32::MAX;
    let mut max_z = i32::MIN;

    for &(x, y, z) in block_map.keys() {
        min_x = min_x.min(x);
        max_x = max_x.max(x);
        min_y = min_y.min(y);
        max_y = max_y.max(y);
        min_z = min_z.min(z);
        max_z = max_z.max(z);
    }

    let width = (max_x - min_x + 1) as i16;
    let height = (max_y - min_y + 1) as i16;
    let length = (max_z - min_z + 1) as i16;

    // Build Palette
    let mut palette: HashMap<String, i32> = HashMap::new();
    palette.insert("minecraft:air".into(), 0);
    let mut next_id = 1i32;

    for state in block_map.values() {
        if !palette.contains_key(state) {
            palette.insert(state.clone(), next_id);
            next_id += 1;
        }
    }

    let palette_max = palette.len() as i32;

    // Serialize BlockData (y * Length + z) * Width + x
    let air_str = "minecraft:air".to_string();
    let mut block_data_bytes = Vec::new();

    for y in min_y..=max_y {
        for z in min_z..=max_z {
            for x in min_x..=max_x {
                let state = block_map.get(&(x, y, z)).unwrap_or(&air_str);
                let id = *palette.get(state).unwrap_or(&0) as u32;
                write_varint(id, &mut block_data_bytes);
            }
        }
    }

    let block_data_i8: Vec<i8> = block_data_bytes.into_iter().map(|b| b as i8).collect();

    Ok(SpongeSchematic {
        version: 2,
        data_version: options.data_version,
        width,
        height,
        length,
        offset: IntArray::new(vec![min_x, min_y, min_z]),
        palette_max,
        palette,
        block_data: ByteArray::new(block_data_i8),
    })
}

pub fn save_schematic<P: AsRef<Path>>(
    schematic: &SpongeSchematic,
    output_path: P,
) -> io::Result<()> {
    let nbt_bytes = fastnbt::to_bytes(schematic)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("NBT error: {}", e)))?;

    let file = File::create(output_path)?;
    let mut encoder = GzEncoder::new(file, Compression::default());
    encoder.write_all(&nbt_bytes)?;
    encoder.finish()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nbs::NbsNote;
    use flate2::read::GzDecoder;
    use std::io::Read;

    #[test]
    fn test_schematic_generation_and_nbt_roundtrip() {
        let song = NbsSong {
            version: 5,
            default_instruments: 16,
            song_length: 20,
            layer_count: 2,
            song_name: "Test Song".into(),
            song_author: "Morro".into(),
            song_original_author: "".into(),
            song_description: "Unit test".into(),
            tempo: 1000,
            time_signature: 4,
            notes: vec![
                NbsNote {
                    tick: 0,
                    layer: 0,
                    instrument: 0, // Harp
                    key: 45,        // F#4 -> note 12
                    velocity: 100,
                    panning: 100,
                    pitch: 0,
                },
                NbsNote {
                    tick: 0,
                    layer: 1,
                    instrument: 1, // Basedrum
                    key: 33,        // F#3 -> note 0
                    velocity: 100,
                    panning: 100,
                    pitch: 0,
                },
                NbsNote {
                    tick: 4,
                    layer: 0,
                    instrument: 6, // Bell
                    key: 57,        // F#5 -> note 24
                    velocity: 100,
                    panning: 100,
                    pitch: 0,
                },
            ],
        };

        let opts = BuildOptions::default();
        let schem = build_schematic(&song, &opts).expect("Build schematic failed");

        assert_eq!(schem.version, 2);
        assert_eq!(schem.data_version, 3953);
        assert!(schem.width > 0);
        assert_eq!(schem.height, 3);
        assert!(schem.length > 0);

        // Check palette contains modern blockstates
        assert!(schem.palette.contains_key("minecraft:air"));
        assert!(schem.palette.contains_key("minecraft:smooth_stone"));
        assert!(schem.palette.contains_key("minecraft:note_block[instrument=harp,note=12,powered=false]"));
        assert!(schem.palette.contains_key("minecraft:note_block[instrument=basedrum,note=0,powered=false]"));
        assert!(schem.palette.contains_key("minecraft:note_block[instrument=bell,note=24,powered=false]"));
        assert!(schem.palette.contains_key("minecraft:gold_block"));
        assert!(schem.palette.contains_key("minecraft:oak_planks"));

        // Roundtrip to GZipped NBT
        let temp_dir = std::env::temp_dir();
        let test_schem_path = temp_dir.join("test_song.schem");
        save_schematic(&schem, &test_schem_path).expect("Save failed");

        // Verify file was written and can be gunzipped
        let gz_bytes = std::fs::read(&test_schem_path).expect("Read schem failed");
        assert!(gz_bytes.len() > 100);

        let mut decoder = GzDecoder::new(&gz_bytes[..]);
        let mut decompressed = Vec::new();
        decoder.read_to_end(&mut decompressed).expect("Gz decode failed");

        // FastNBT parse verify
        let parsed_schem: fastnbt::Value = fastnbt::from_bytes(&decompressed).expect("NBT decode failed");
        match parsed_schem {
            fastnbt::Value::Compound(map) => {
                assert!(map.contains_key("Version"));
                assert!(map.contains_key("DataVersion"));
                assert!(map.contains_key("Palette"));
                assert!(map.contains_key("BlockData"));
            }
            _ => panic!("Expected compound NBT tag"),
        }

        let _ = std::fs::remove_file(test_schem_path);
    }
}

