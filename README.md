# nbs2schem

High-performance CLI tool to convert **Note Block Studio** (`.nbs`), **MIDI** (`.mid`, `.midi`), and **Minecraft Vanilla Structure** (`.nbt`) files into modern Minecraft (1.21+ / 26.3) **Sponge Schematic** (`.schem`) format.

Fully compatible with **WorldEdit**, **FAWE** (FastAsyncWorldEdit), and **Axiom**.

---

[English](#english) | [Türkçe](#türkçe)

---

## English

### Overview

Classic Note Block Studio exports in the legacy MCEdit `.schematic` format using numeric Block IDs (e.g., 25, 35). Modern Minecraft (1.13+ Flattening up to 26.3) does not support numeric IDs and expects modern string Block States (`minecraft:note_block[instrument=...,note=...]`) packed into Sponge Schematic v2 NBT.

`nbs2schem` bridges this gap with zero-loss, high-performance conversion directly into modern Minecraft schematics.

### Features

- **Triple Format Support:** Automatically detects and converts Note Block Studio (`.nbs` v0–v5), Standard MIDI files (`.mid`, `.midi`), and Vanilla Minecraft Structure Blocks (`.nbt`).
- **16 Modern Vanilla Instruments:** Full mapping for Harp, Basedrum, Snare, Hat, Bass, Flute, Bell, Guitar, Chime, Xylophone, Iron Xylophone, Cowbell, Didgeridoo, Bit, Banjo, and Pling. Automatically places the correct physical foundation blocks (Gold Block, Bone Block, Wool, Ice, etc.) beneath each note block.
- **Intelligent MIDI Importer:** Maps General MIDI channels and program changes (Pianos, Guitars, Bass, Flutes, Bells, Percussion) to Minecraft instruments and transposes all pitches into Minecraft's 2-octave range ($F\sharp3$ – $F\sharp5$).
- **Zero-Collision Spine Circuit Architecture:**
  - Signal travels down a central repeater spine ($Z$-axis) and branches out into lateral rib stations ($X$-axis).
  - Consecutive tick stations alternate sides ($+X$ and $-X$), making signal crosstalk or block overlap impossible even for 1-tick delay chords.
  - Guaranteed air headroom above every note block so sounds are never muffled.
- **Built-in Start Button:** An oak button on a stone block (`minecraft:oak_button`) is automatically placed at the front of the track for instant one-click playback.
- **Speed & Tempo Scaling:** Scale playback speed on the fly using `--speed` (e.g., `1.5` or `2.0`). For `.nbt` structures, it dynamically rewrites repeater delays; for `.nbs`/`.mid`, it rescales tick timing.
- **Ultra Fast:** Written in pure Rust with `fastnbt` and `flate2`. Converts dense orchestral scores with hundreds of notes in single-digit milliseconds.

### Installation & Build

Requires [Rust and Cargo](https://rustup.rs/):

```bash
git clone https://github.com/ibrahimhalilsezgin/nbs2schem.git
cd nbs2schem
cargo build --release
```

Executable output: `target/release/nbs2schem.exe` (or `target/release/nbs2schem` on Linux/macOS).

### Usage

```bash
# Convert a Note Block Studio (.nbs) file
nbs2schem song.nbs

# Convert a Standard MIDI (.mid / .midi) file
nbs2schem song.mid -o song.schem

# Convert a Minecraft Vanilla Structure (.nbt) file
nbs2schem structure.nbt -o song.schem

# Inspect song metadata, notes, and instrument breakdown without exporting
nbs2schem song.mid --info

# Adjust playback speed (e.g. 1.5x or 2.0x faster)
nbs2schem song.mid --speed 1.5

# Specify target Minecraft DataVersion (default: 3953 for modern MC / 26.3)
nbs2schem song.nbs --data-version 3953
```

### In-Game Usage

1. Copy the generated `.schem` file to your server/client schematic directory:
   - WorldEdit: `config/worldedit/schematics/`
   - FAWE: `plugins/FastAsyncWorldEdit/schematics/`
   - Axiom: `.minecraft/axiom/schematics/`
2. In-game, run: `//schem load <filename>`
3. Stand at your target location and run: `//paste`
4. Press the oak button at the front of the track to start the song!

---

## Türkçe

### Genel Bakış

Klasik Note Block Studio, eski sayısal Block ID (Alpha MCEdit) şematik formatında çıktı verir. Modern Minecraft (1.13+ Flattening ve 26.3) sayısal ID'leri desteklemez; modern Blok Durumları (`minecraft:note_block[instrument=...,note=...]`) ve Sponge Schematic v2 NBT formatı bekler.

`nbs2schem`, `.nbs`, `.mid` ve `.nbt` dosyalarını doğrudan modern Minecraft standartlarında devrelere dönüştürür.

### Özellikler

- **Üçlü Format Desteği:** Note Block Studio (`.nbs` v0-v5), Standart MIDI (`.mid`, `.midi`) ve Minecraft Vanilla Structure (`.nbt`) dosyalarını otomatik algılar ve dönüştürür.
- **16 Modern Vanilla Enstrümanı:** Harp, Basedrum, Snare, Hat, Bass, Flute, Bell, Guitar, Chime, Xylophone, Iron Xylophone, Cowbell, Didgeridoo, Bit, Banjo, Pling. Nota bloklarının altına doğru fiziksel zemin bloklarını (Altın, Kemik, Yün vb.) otomatik yerleştirir.
- **Akıllı MIDI İçe Aktarma:** MIDI kanallarını ve enstrüman programlarını (Piyano, Gitar, Bas, Flüt, Çan, Davul) Minecraft'ın 16 vanilla enstrümanına eşler; notaları 2 oktavlık (F#3 - F#5) nota bloğu aralığına otomatik transpoze eder.
- **Sıfır Çakışmalı Omurga Devresi:**
  - Tekrarlayıcı omurgası (Z ekseni) ve iki yana sıralı kaburga (X ekseni) mimarisi.
  - Ardışık tick istasyonları sağ ve sol kanatlara dönüşümlü dağıtılır (`+X` ve `-X`); 1 redstone tick gecikmeli akorlarda dahi redstone sinyal karışması ve blok çakışması imkansızdır.
  - Her nota bloğunun üzerinde doğrudan ses çıkışı için hava boşluğu garanti edilir.
- **Otomatik Başlangıç Düğmesi:** Devrenin başına tek tıkla şarkıyı başlatan taş blok üzeri meşe düğme (`minecraft:oak_button`) entegre edilmiştir.
- **Hızlandırma / Yavaşlatma:** `--speed` parametresiyle devre hızını (örneğin `1.5` veya `2.0` kat) anında ölçeklendirir.
- **Süper Hızlı:** Rust ile yazılmıştır; saniyede on binlerce notayı milisaniyeler içinde dönüştürür.

### Kullanım

```bash
# Note Block Studio (.nbs) dönüştürme
nbs2schem sarki.nbs

# MIDI (.mid / .midi) dosyasını doğrudan şematiğe çevirme
nbs2schem sarki.mid -o sarki.schem

# Minecraft Vanilla Structure (.nbt) dosyasını şematiğe çevirme
nbs2schem yapi.nbt -o yapi.schem

# Şarkı veya yapı analizi (bilgi dökümü)
nbs2schem sarki.mid --info

# Hız çarpanı (örneğin 1.5 kat hız veya 2 kat hız)
nbs2schem sarki.mid --speed 1.5

# Hedef Minecraft DataVersion belirleme (varsayılan: 3953 - 1.21+ / 26.3)
nbs2schem sarki.nbs --data-version 3953
```

---

## License

MIT License. Open source and free to use.
