# nbs2schem

Modern Minecraft (1.21+ / 26.3) için Note Block Studio (`.nbs`), MIDI (`.mid`) ve Vanilla Structure (`.nbt`) -> Sponge Schematic (`.schem`) dönüştürücü CLI aracı.

Klasik Note Block Studio'nun eski sayısal Block ID (Alpha MCEdit) şematik formatı veya Vanilla Structure Block (`.nbt`) dosyaları yerine, modern Minecraft'ın Block State palet sistemini (`minecraft:note_block[instrument=...,note=...]`) ve Sponge v2 NBT formatını kullanır. WorldEdit, FAWE ve Axiom ile %100 uyumludur.

## Özellikler

- **Üçlü Format Desteği:** Note Block Studio (`.nbs` v0-v5), Standart MIDI (`.mid`, `.midi`) ve Minecraft Vanilla Structure (`.nbt`) dosyalarını otomatik algılar ve dönüştürür.
- **Akıllı MIDI İçe Aktarma:** MIDI kanallarını ve enstrüman programlarını (Piyano, Gitar, Bas, Flüt, Çan, Davul) Minecraft'ın 16 vanilla enstrümanına eşler; notaları 2 oktavlık (F#3 - F#5) nota bloğu aralığına otomatik transpoze eder.
- **16 Modern Vanilla Enstrümanı:** Harp, Basedrum, Snare, Hat, Bass, Flute, Bell, Guitar, Chime, Xylophone, Iron Xylophone, Cowbell, Didgeridoo, Bit, Banjo, Pling. Nota bloklarının altına doğru fiziksel zemin bloklarını (Altın, Kemik, Yün vb.) otomatik yerleştirir.
- **Sıfır Çakışmalı Omurga Devresi:**
  - Tekrarlayıcı omurgası (Z ekseni) ve iki yana sıralı kaburga (X ekseni) mimarisi.
  - Ardışık tick istasyonları sağ ve sol kanatlara dönüşümlü dağıtılır (`+X` ve `-X`); 1 redstone tick gecikmeli akorlarda dahi redstone sinyal karışması ve blok çakışması imkansızdır.
  - Her nota bloğunun üzerinde doğrudan ses çıkışı için hava boşluğu garanti edilir.
- **Otomatik Başlangıç Düğmesi:** Devrenin başına tek tıkla şarkıyı başlatan taş blok üzeri meşe düğme (`minecraft:oak_button`) entegre edilmiştir.
- **Süper Hızlı:** Rust ile yazılmıştır; saniyede on binlerce notayı milisaniyeler içinde dönüştürür.

## Kurulum & Derleme

```bash
cargo build --release
```

Çalıştırılabilir dosya: `target/release/nbs2schem.exe`

## Kullanım

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

## Minecraft İçinde Kullanım

1. Oluşan `.schem` dosyasını sunucunuzun veya istemcinizin `plugins/FastAsyncWorldEdit/schematics/`, `config/worldedit/schematics/` veya Axiom klasörüne atın.
2. Oyun içinde `//schem load <dosya_adi>` yazın.
3. Uygun bir konuma gelip `//paste` komutunu çalıştırın.
4. Devrenin başındaki düğmeye basın, şarkı çalsın!
