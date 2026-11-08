mod midi;
mod nbs;
mod schematic;
mod structure;

use clap::Parser;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::process;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(
    name = "nbs2schem",
    about = "Converts Note Block Studio (.nbs), MIDI (.mid), or Vanilla Structure (.nbt) to modern Minecraft 26.3 / 1.21+ Sponge Schematic (.schem)",
    version = "0.3.0"
)]
struct Args {
    /// Path to input .nbs, .mid, or .nbt file
    #[arg(value_name = "FILE")]
    input: PathBuf,

    /// Path to output .schem file (defaults to <input>.schem)
    #[arg(short, long, value_name = "OUTPUT")]
    output: Option<PathBuf>,

    /// Target Minecraft DataVersion (default: 3953 for modern MC / 26.3)
    #[arg(long, default_value_t = 3953)]
    data_version: i32,

    /// Tempo / Speed scaling factor (e.g. 2.0 for 2x faster, 0.5 for half speed)
    #[arg(long, visible_alias = "speed", default_value_t = 1.0)]
    tempo_scale: f64,

    /// Display file information and exit
    #[arg(short, long)]
    info: bool,
}

fn main() {
    let args = Args::parse();

    if !args.input.exists() {
        eprintln!("Error: File not found: {}", args.input.display());
        process::exit(1);
    }

    let raw_data = match fs::read(&args.input) {
        Ok(d) => d,
        Err(e) => {
            eprintln!("Error: Cannot read input file: {}", e);
            process::exit(1);
        }
    };

    let is_midi = args
        .input
        .extension()
        .map(|ext| ext.eq_ignore_ascii_case("mid") || ext.eq_ignore_ascii_case("midi"))
        .unwrap_or(false)
        || raw_data.starts_with(b"MThd");

    let is_nbt = !is_midi
        && (args
            .input
            .extension()
            .map(|ext| ext.eq_ignore_ascii_case("nbt"))
            .unwrap_or(false)
            || raw_data.starts_with(&[0x1F, 0x8B]));

    let output_path = match args.output {
        Some(p) => p,
        None => {
            let mut p = args.input.clone();
            p.set_extension("schem");
            p
        }
    };

    if is_nbt {
        println!("=== Format Detected: Minecraft Vanilla Structure (.nbt) ===");
        println!("Input:       {}", args.input.display());

        let start_time = Instant::now();
        let schem = match structure::convert_structure_to_schematic(&raw_data, args.tempo_scale) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Failed to convert Structure NBT: {}", e);
                process::exit(1);
            }
        };

        println!(
            "Dimensions:  Width(X)={} Height(Y)={} Length(Z)={}",
            schem.width, schem.height, schem.length
        );
        println!("Palette:     {} unique block states", schem.palette_max);
        println!("DataVersion: {} (Modern Minecraft 26.3 compatible)", schem.data_version);

        if args.info {
            println!("\nBlock Palette Preview (first 20):");
            let mut items: Vec<(&String, &i32)> = schem.palette.iter().collect();
            items.sort_by_key(|a| a.1);
            for (state, id) in items.iter().take(20) {
                println!("  [{:>2}] {}", id, state);
            }
            if items.len() > 20 {
                println!("  ... and {} more states", items.len() - 20);
            }
            return;
        }

        if let Err(e) = schematic::save_schematic(&schem, &output_path) {
            eprintln!("Error: Failed to save schematic: {}", e);
            process::exit(1);
        }

        let elapsed = start_time.elapsed();
        println!(
            "\nSuccess: Exported to {} in {:.2}ms",
            output_path.display(),
            elapsed.as_secs_f64() * 1000.0
        );
        println!("Compatible with: WorldEdit, FAWE, Axiom, Minecraft 26.3 / 1.21+");
        return;
    }

    // Parse as MIDI or NBS
    let song = if is_midi {
        println!("=== Format Detected: Standard MIDI File (.mid) ===");
        match midi::parse_midi(&raw_data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Failed to parse MIDI file: {}", e);
                process::exit(1);
            }
        }
    } else {
        println!("=== Format Detected: Note Block Studio (.nbs) ===");
        match nbs::parse_nbs(&raw_data) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Error: Failed to parse NBS file: {}", e);
                process::exit(1);
            }
        }
    };

    let title = if song.song_name.is_empty() {
        args.input
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
    } else {
        &song.song_name
    };

    let author = if song.song_author.is_empty() {
        if song.song_original_author.is_empty() {
            "Unknown"
        } else {
            &song.song_original_author
        }
    } else {
        &song.song_author
    };

    let tps = if song.tempo > 0 {
        song.tempo as f64 / 100.0
    } else {
        10.0
    };
    let duration_sec = if tps > 0.0 {
        (song.song_length as f64) / tps
    } else {
        0.0
    };

    println!("Song:        {}", title);
    println!("Author:      {}", author);
    println!("Tempo:       {:.2} TPS", tps);
    println!("Length:      {} ticks (~{:.1}s)", song.song_length, duration_sec);
    println!("Total Notes: {}", song.notes.len());

    if args.info {
        let mut inst_counts: HashMap<u8, usize> = HashMap::new();
        for n in &song.notes {
            *inst_counts.entry(n.instrument).or_insert(0) += 1;
        }
        let mut sorted_inst: Vec<(u8, usize)> = inst_counts.into_iter().collect();
        sorted_inst.sort_by(|a, b| b.1.cmp(&a.1));

        println!("\nInstrument Breakdown:");
        for (inst, count) in sorted_inst {
            let (name, under_block) = schematic::get_instrument_info(inst);
            println!("  - [{:>2}] {:<14} ({:<22}): {} notes", inst, name, under_block, count);
        }
        return;
    }

    println!("\nBuilding schematic for modern Minecraft (DataVersion {})...", args.data_version);
    let start_time = Instant::now();

    let build_opts = schematic::BuildOptions {
        data_version: args.data_version,
        tempo_scale: args.tempo_scale,
    };

    let schem = match schematic::build_schematic(&song, &build_opts) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: Failed to build schematic: {}", e);
            process::exit(1);
        }
    };

    println!(
        "Dimensions:  Width(X)={} Height(Y)={} Length(Z)={}",
        schem.width, schem.height, schem.length
    );
    println!("Palette:     {} unique block states", schem.palette_max);

    if let Err(e) = schematic::save_schematic(&schem, &output_path) {
        eprintln!("Error: Failed to save schematic: {}", e);
        process::exit(1);
    }

    let elapsed = start_time.elapsed();
    println!(
        "Success: Exported to {} in {:.2}ms",
        output_path.display(),
        elapsed.as_secs_f64() * 1000.0
    );
    println!("Compatible with: WorldEdit, FAWE, Axiom, Minecraft 26.3 / 1.21+");
}
