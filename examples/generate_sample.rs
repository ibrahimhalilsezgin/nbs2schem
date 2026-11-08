use byteorder::{LittleEndian, WriteBytesExt};
use std::fs::File;
use std::io::{self, Write};
use std::path::Path;

fn write_str<W: Write>(w: &mut W, s: &str) -> io::Result<()> {
    w.write_u32::<LittleEndian>(s.len() as u32)?;
    w.write_all(s.as_bytes())
}

fn main() -> io::Result<()> {
    let output_path = Path::new("sample_song.nbs");
    let mut file = File::create(output_path)?;

    // Modern NBS (v5)
    file.write_u16::<LittleEndian>(0)?; // signals new format
    file.write_u8(5)?; // v5
    file.write_u8(16)?; // 16 vanilla instruments
    file.write_u16::<LittleEndian>(32)?; // song length in ticks
    file.write_u16::<LittleEndian>(2)?; // layer count

    write_str(&mut file, "Morro Groove")?;
    write_str(&mut file, "İbrahim & Morro")?;
    write_str(&mut file, "")?;
    write_str(&mut file, "Demo song for modern Minecraft 26.3 schematic")?;
    file.write_u16::<LittleEndian>(1000)?; // 10.0 TPS
    file.write_u8(0)?; // auto save
    file.write_u8(10)?;
    file.write_u8(4)?; // 4/4
    file.write_u32::<LittleEndian>(0)?;
    file.write_u32::<LittleEndian>(0)?;
    file.write_u32::<LittleEndian>(0)?;
    file.write_u32::<LittleEndian>(0)?;
    file.write_u32::<LittleEndian>(0)?;
    write_str(&mut file, "")?; // midi file

    // Loop fields
    file.write_u8(0)?;
    file.write_u8(0)?;
    file.write_u16::<LittleEndian>(0)?;

    // Notes: A simple funky bass + melody rhythm
    // (delta_tick, inst, key)
    let notes = [
        // Tick 0 (jump 1 from -1)
        (1, 1, 33), // kick
        (0, 4, 38), // bass
        (0, 0, 45), // harp
        // Tick 2 (jump 2)
        (2, 3, 40), // hat
        (0, 0, 48), // harp
        // Tick 4 (jump 2)
        (2, 2, 35), // snare
        (0, 4, 41), // bass
        (0, 6, 50), // bell
        // Tick 6 (jump 2)
        (2, 3, 40), // hat
        (0, 0, 52), // harp
        // Tick 8 (jump 2)
        (2, 1, 33), // kick
        (0, 4, 38), // bass
        (0, 7, 45), // guitar
        // Tick 12 (jump 4)
        (4, 2, 35), // snare
        (0, 6, 57), // bell
    ];

    let mut first = true;
    for (jump, inst, key) in notes {
        if jump > 0 {
            if !first {
                file.write_u16::<LittleEndian>(0)?; // end layer
            }
            file.write_u16::<LittleEndian>(jump)?;
            file.write_u16::<LittleEndian>(1)?; // layer 0
            first = false;
        } else {
            file.write_u16::<LittleEndian>(1)?; // next layer
        }
        file.write_u8(inst)?;
        file.write_u8(key)?;
        file.write_u8(100)?; // vel
        file.write_u8(100)?; // pan
        file.write_i16::<LittleEndian>(0)?; // pitch
    }
    file.write_u16::<LittleEndian>(0)?; // end layers
    file.write_u16::<LittleEndian>(0)?; // end notes

    println!("Created sample_song.nbs successfully!");
    Ok(())
}
