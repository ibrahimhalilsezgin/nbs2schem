use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{self, Cursor, Read};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NbsNote {
    pub tick: u32,
    pub layer: u16,
    pub instrument: u8,
    pub key: u8, // 33-57 (piano key, 33 = F#3 = note 0)
    pub velocity: u8,
    pub panning: u8,
    pub pitch: i16,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct NbsSong {
    pub version: u8,
    pub default_instruments: u8,
    pub song_length: u16,
    pub layer_count: u16,
    pub song_name: String,
    pub song_author: String,
    pub song_original_author: String,
    pub song_description: String,
    pub tempo: u16, // hundredths of TPS, e.g. 1000 = 10.0 TPS
    pub time_signature: u8,
    pub notes: Vec<NbsNote>,
}

fn read_string<R: Read>(reader: &mut R) -> io::Result<String> {
    let len = reader.read_u32::<LittleEndian>()? as usize;
    if len == 0 {
        return Ok(String::new());
    }
    if len > 500_000 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "String length too large in NBS file",
        ));
    }
    let mut buf = vec![0u8; len];
    reader.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

pub fn parse_nbs(data: &[u8]) -> io::Result<NbsSong> {
    let mut reader = Cursor::new(data);

    let first_two = reader.read_u16::<LittleEndian>()?;
    let (version, default_instruments, song_length) = if first_two == 0 {
        // Modern NBS (v1-v5)
        let ver = reader.read_u8()?;
        let inst_count = reader.read_u8()?;
        let len = reader.read_u16::<LittleEndian>()?;
        (ver, inst_count, len)
    } else {
        // Classic NBS (v0)
        (0u8, 10u8, first_two)
    };

    let layer_count = reader.read_u16::<LittleEndian>()?;
    let song_name = read_string(&mut reader)?;
    let song_author = read_string(&mut reader)?;
    let song_original_author = read_string(&mut reader)?;
    let song_description = read_string(&mut reader)?;
    let tempo = reader.read_u16::<LittleEndian>()?;
    let _auto_save = reader.read_u8()?;
    let _auto_save_duration = reader.read_u8()?;
    let time_signature = reader.read_u8()?;
    let _minutes_spent = reader.read_u32::<LittleEndian>()?;
    let _left_clicks = reader.read_u32::<LittleEndian>()?;
    let _right_clicks = reader.read_u32::<LittleEndian>()?;
    let _blocks_added = reader.read_u32::<LittleEndian>()?;
    let _blocks_removed = reader.read_u32::<LittleEndian>()?;
    let _midi_or_schem = read_string(&mut reader)?;

    if version >= 4 {
        let _loop_on = reader.read_u8()?;
        let _max_loop = reader.read_u8()?;
        let _loop_start = reader.read_u16::<LittleEndian>()?;
    }

    let mut notes = Vec::new();
    let mut current_tick: i64 = -1;

    loop {
        let tick_jumps = match reader.read_u16::<LittleEndian>() {
            Ok(v) => v,
            Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e),
        };

        if tick_jumps == 0 {
            break;
        }

        current_tick += tick_jumps as i64;
        let mut current_layer: i64 = -1;

        loop {
            let layer_jumps = reader.read_u16::<LittleEndian>()?;
            if layer_jumps == 0 {
                break;
            }

            current_layer += layer_jumps as i64;
            let instrument = reader.read_u8()?;
            let key = reader.read_u8()?;

            let (velocity, panning, pitch) = if version >= 4 {
                let v = reader.read_u8()?;
                let p = reader.read_u8()?;
                let pt = reader.read_i16::<LittleEndian>()?;
                (v, p, pt)
            } else {
                (100, 100, 0)
            };

            notes.push(NbsNote {
                tick: current_tick.max(0) as u32,
                layer: current_layer.max(0) as u16,
                instrument,
                key,
                velocity,
                panning,
                pitch,
            });
        }
    }

    Ok(NbsSong {
        version,
        default_instruments,
        song_length,
        layer_count,
        song_name,
        song_author,
        song_original_author,
        song_description,
        tempo,
        time_signature,
        notes,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::{LittleEndian, WriteBytesExt};

    #[test]
    fn test_parse_v5_nbs_binary() {
        let mut buf = Vec::new();
        // New format header
        buf.write_u16::<LittleEndian>(0).unwrap(); // song_length = 0 signals new format
        buf.write_u8(5).unwrap(); // version 5
        buf.write_u8(16).unwrap(); // vanilla inst count
        buf.write_u16::<LittleEndian>(10).unwrap(); // song_length
        buf.write_u16::<LittleEndian>(1).unwrap(); // layer_count

        // String helper
        let write_str = |b: &mut Vec<u8>, s: &str| {
            b.write_u32::<LittleEndian>(s.len() as u32).unwrap();
            b.extend_from_slice(s.as_bytes());
        };

        write_str(&mut buf, "Megalovania"); // song_name
        write_str(&mut buf, "Toby Fox"); // song_author
        write_str(&mut buf, ""); // song_orig_author
        write_str(&mut buf, "Test description"); // song_desc
        buf.write_u16::<LittleEndian>(1000).unwrap(); // tempo 10 TPS
        buf.write_u8(0).unwrap(); // auto_save
        buf.write_u8(10).unwrap(); // auto_save_dur
        buf.write_u8(4).unwrap(); // time_signature
        buf.write_u32::<LittleEndian>(0).unwrap(); // min_spent
        buf.write_u32::<LittleEndian>(0).unwrap(); // left_clicks
        buf.write_u32::<LittleEndian>(0).unwrap(); // right_clicks
        buf.write_u32::<LittleEndian>(0).unwrap(); // blocks_added
        buf.write_u32::<LittleEndian>(0).unwrap(); // blocks_removed
        write_str(&mut buf, ""); // midi/schem name
        // v4+ loop fields
        buf.write_u8(0).unwrap(); // loop_on
        buf.write_u8(0).unwrap(); // max_loop
        buf.write_u16::<LittleEndian>(0).unwrap(); // loop_start

        // Note section
        // Tick 0 (first tick jump = 1, since starting from -1)
        buf.write_u16::<LittleEndian>(1).unwrap();
        // Layer 0 (first layer jump = 1, starting from -1)
        buf.write_u16::<LittleEndian>(1).unwrap();
        buf.write_u8(0).unwrap(); // inst = harp
        buf.write_u8(45).unwrap(); // key = 45 (F#4)
        // v4+ note fields
        buf.write_u8(100).unwrap(); // velocity
        buf.write_u8(100).unwrap(); // panning
        buf.write_i16::<LittleEndian>(0).unwrap(); // pitch

        // End layers for tick 0
        buf.write_u16::<LittleEndian>(0).unwrap();

        // End notes section
        buf.write_u16::<LittleEndian>(0).unwrap();

        let song = parse_nbs(&buf).expect("Parse failed");
        assert_eq!(song.version, 5);
        assert_eq!(song.song_name, "Megalovania");
        assert_eq!(song.song_author, "Toby Fox");
        assert_eq!(song.tempo, 1000);
        assert_eq!(song.notes.len(), 1);
        assert_eq!(song.notes[0].tick, 0);
        assert_eq!(song.notes[0].instrument, 0);
        assert_eq!(song.notes[0].key, 45);
    }
}

