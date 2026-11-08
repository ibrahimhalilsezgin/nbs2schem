use crate::nbs::{NbsNote, NbsSong};
use midly::{MetaMessage, MidiMessage, Smf, Timing, TrackEventKind};
use std::io;

pub fn parse_midi(data: &[u8]) -> io::Result<NbsSong> {
    let smf = Smf::parse(data)
        .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, format!("MIDI parse error: {}", e)))?;

    let ppq = match smf.header.timing {
        Timing::Metrical(ticks) => ticks.as_int() as f64,
        Timing::Timecode(fps, sub) => (fps.as_f32() * sub as f32) as f64,
    };

    let mut song_name = String::new();
    let mut notes = Vec::new();

    // Default tempo: 120 BPM = 500,000 microseconds per beat
    let mut micros_per_beat = 500_000.0;
    let target_tps = 10.0; // 10 redstone ticks per second

    // First scan for track name
    for track in &smf.tracks {
        for event in track {
            if let TrackEventKind::Meta(MetaMessage::TrackName(name_bytes)) = event.kind {
                let name = String::from_utf8_lossy(name_bytes).trim().to_string();
                if !name.is_empty() && song_name.is_empty() {
                    song_name = name;
                }
            }
        }
    }

    if song_name.is_empty() {
        song_name = "İstiklal Marşı".to_string();
    }

    for (track_idx, track) in smf.tracks.iter().enumerate() {
        let mut current_time_sec = 0.0;
        let mut channel_program = [0u8; 16];

        for event in track {
            let delta = event.delta.as_int() as f64;
            let seconds_per_tick = (micros_per_beat / 1_000_000.0) / ppq;
            current_time_sec += delta * seconds_per_tick;

            match event.kind {
                TrackEventKind::Meta(MetaMessage::Tempo(tempo_micros)) => {
                    micros_per_beat = tempo_micros.as_int() as f64;
                }
                TrackEventKind::Midi { channel, message } => {
                    let ch = channel.as_int();
                    match message {
                        MidiMessage::ProgramChange { program } => {
                            channel_program[ch as usize] = program.as_int();
                        }
                        MidiMessage::NoteOn { key, vel } => {
                            if vel.as_int() > 0 {
                                let raw_key = key.as_int();
                                let inst = if ch == 9 {
                                    match raw_key {
                                        35 | 36 => 1, // Basedrum
                                        38 | 40 => 2, // Snare
                                        42 | 44 | 46 | 49 | 51 => 3, // Hat
                                        56 => 11,     // Cowbell
                                        _ => 1,
                                    }
                                } else {
                                    match channel_program[ch as usize] {
                                        0..=7 => 0,     // Piano -> Harp
                                        24..=31 => 7,   // Guitar
                                        32..=39 => 4,   // Bass
                                        8..=9 => 6,     // Bell
                                        10..=12 => 9,   // Xylophone
                                        13..=15 => 8,   // Chime
                                        72..=79 => 5,   // Flute
                                        80..=87 => 13,  // Bit
                                        105 => 14,      // Banjo
                                        112..=119 => 15,// Pling
                                        _ => 0,         // Harp
                                    }
                                };

                                // Map MIDI key (F#3 = 54 to F#5 = 78) into Minecraft 0..24
                                let mut shifted_key = raw_key as i32;
                                while shifted_key < 54 {
                                    shifted_key += 12;
                                }
                                while shifted_key > 78 {
                                    shifted_key -= 12;
                                }
                                let pitch = (shifted_key - 54).clamp(0, 24) as u8;
                                let nbs_key = pitch + 33;

                                let song_tick = (current_time_sec * target_tps).round() as u32;

                                notes.push(NbsNote {
                                    tick: song_tick,
                                    layer: track_idx as u16,
                                    instrument: inst,
                                    key: nbs_key,
                                    velocity: vel.as_int(),
                                    panning: 100,
                                    pitch: 0,
                                });
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }

    notes.sort_by_key(|n| n.tick);
    let max_tick = notes.last().map(|n| n.tick).unwrap_or(0);

    Ok(NbsSong {
        version: 5,
        default_instruments: 16,
        song_length: max_tick as u16,
        layer_count: smf.tracks.len() as u16,
        song_name,
        song_author: "Osman Zeki Üngör / Mehmet Akif Ersoy".into(),
        song_original_author: "".into(),
        song_description: "İstiklal Marşı - Turkish National Anthem".into(),
        tempo: (target_tps * 100.0) as u16,
        time_signature: 4,
        notes,
    })
}
