// src/map/parser.rs
// ==============================================================================
// Rusthia — Parser de maps
// Traduction complète de MapParser.cs
//
// Formats supportés :
//   .phxm  → ZIP (metadata.json + objects.phxmo)         [MapParser.PHXM()]
//   .sspm  → Binaire v1 et v2                            [MapParser.SSPM()]
//   .txt   → SSMapV1 (format texte CSV legacy)           [MapParser.SSMapV1()]
// ==============================================================================

use std::io::{Cursor, Read, Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use zip::ZipArchive;
use super::types::{MapData, NoteData, PhxmMetadata};

// ==============================================================================
// ERREURS
// ==============================================================================

#[derive(Debug)]
pub enum ParseError {
    InvalidSignature(String),
    UnsupportedVersion(u16),
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    MissingEntry(String),
    InvalidFormat(String),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidSignature(s) => write!(f, "Signature invalide: {}", s),
            Self::UnsupportedVersion(v) => write!(f, "Version non supportée: {}", v),
            Self::Io(e) => write!(f, "Erreur I/O: {}", e),
            Self::Zip(e) => write!(f, "Erreur ZIP: {}", e),
            Self::Json(e) => write!(f, "Erreur JSON: {}", e),
            Self::MissingEntry(s) => write!(f, "Entrée manquante dans l'archive: {}", s),
            Self::InvalidFormat(s) => write!(f, "Format invalide: {}", s),
        }
    }
}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self { Self::Io(e) }
}
impl From<zip::result::ZipError> for ParseError {
    fn from(e: zip::result::ZipError) -> Self { Self::Zip(e) }
}
impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self { Self::Json(e) }
}

// ==============================================================================
// POINT D'ENTRÉE
// ==============================================================================

/// Détecter le format et parser la map.
/// Équivalent de `MapParser.Decode()` avec dispatch par extension.
pub fn parse_map(bytes: &[u8], filename: &str) -> Result<MapData, ParseError> {
    let ext = filename.rsplit('.').next().unwrap_or("").to_lowercase();

    match ext.as_str() {
        "phxm" => parse_phxm(bytes),
        "sspm" => parse_sspm(bytes),
        "txt"  => parse_ssmap_v1_bytes(bytes),
        other  => Err(ParseError::InvalidFormat(format!("Extension non supportée: {}", other))),
    }
}

// ==============================================================================
// FORMAT PHXM — MapParser.PHXM()
// Archive ZIP contenant metadata.json + objects.phxmo + audio + cover
// ==============================================================================

/// Parser un fichier PHXM (format natif Rhythia)
pub fn parse_phxm(bytes: &[u8]) -> Result<MapData, ParseError> {
    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor)?;

    // --- Lire metadata.json ---
    let meta: PhxmMetadata = {
        let mut entry = archive.by_name("metadata.json")
            .map_err(|_| ParseError::MissingEntry("metadata.json".into()))?;
        let mut json = String::new();
        entry.read_to_string(&mut json)?;
        serde_json::from_str(&json)?
    };

    // --- Lire objects.phxmo (notes binaires) ---
    let notes = {
        let mut entry = archive.by_name("objects.phxmo")
            .map_err(|_| ParseError::MissingEntry("objects.phxmo".into()))?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        parse_phxmo_binary(&buf)?
    };

    // --- Lire l'audio si présent ---
    let audio = if meta.has_audio {
        let entry_name = format!("audio.{}", meta.audio_ext);
        let mut entry = archive.by_name(&entry_name)
            .map_err(|_| ParseError::MissingEntry(entry_name))?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        buf
    } else {
        Vec::new()
    };

    // --- Lire la cover si présente ---
    let cover = if meta.has_cover {
        let mut entry = archive.by_name("cover.png")
            .map_err(|_| ParseError::MissingEntry("cover.png".into()))?;
        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        buf
    } else {
        Vec::new()
    };

    Ok(MapData {
        id: meta.id,
        title: meta.title,
        artist: meta.artist,
        mappers: meta.mappers,
        difficulty: meta.difficulty,
        difficulty_name: meta.difficulty_name,
        length_ms: meta.length,
        audio,
        cover,
        audio_ext: meta.audio_ext,
        notes,
    })
}

/// Parser le format binaire des notes PHXMO / objets SSPM.
/// Format par note :
///   uint32  millisecond
///   bool    isQuantum (1 = coordonnées float, 0 = coordonnées entières)
///   if quantum:  float x, float y
///   else:        uint8 x, uint8 y  (remappé: val - 1 ∈ {-1, 0, 1})
///
/// Traduction directe de la boucle dans MapParser.PHXM() L609-628
fn parse_phxmo_binary(data: &[u8]) -> Result<Vec<NoteData>, ParseError> {
    let mut cursor = Cursor::new(data);

    // Header : type_count (ignoré) + note_count
    let _type_count = cursor.read_u32::<LittleEndian>()?;
    let note_count = cursor.read_u32::<LittleEndian>()?;

    let mut notes = Vec::with_capacity(note_count as usize);

    for i in 0..note_count as usize {
        let ms = cursor.read_u32::<LittleEndian>()?;
        let quantum = cursor.read_u8()? != 0;

        let (x, y) = if quantum {
            // Coordonnées float précises (quantum notes)
            let x = cursor.read_f32::<LittleEndian>()?;
            let y = cursor.read_f32::<LittleEndian>()?;
            (x, y)
        } else {
            // Coordonnées entières discrètes : 0,1,2 → -1,0,1
            // L623-624: x = objects.Get(1)[0] - 1; y = objects.Get(1)[0] - 1
            let x = cursor.read_u8()? as f32 - 1.0;
            let y = cursor.read_u8()? as f32 - 1.0;
            (x, y)
        };

        notes.push(NoteData { index: i, millisecond: ms, x, y });
    }

    // Trier par timestamp — Array.Sort(notes) dans le code original
    notes.sort_by_key(|n| n.millisecond);

    // Réindexer après le tri (comme la boucle L374-377 dans MapParser.cs)
    for (i, note) in notes.iter_mut().enumerate() {
        note.index = i;
    }

    Ok(notes)
}

// ==============================================================================
// FORMAT SSPM — MapParser.SSPM()
// Binaire propriétaire SoundSpace+, versions 1 et 2
// Signature: "SS+m" (4 bytes) + uint16 version
// ==============================================================================

pub fn parse_sspm(bytes: &[u8]) -> Result<MapData, ParseError> {
    let mut cursor = Cursor::new(bytes);

    // Vérifier la signature "SS+m" — MapParser.SSPM() L232
    let mut sig = [0u8; 4];
    cursor.read_exact(&mut sig)?;
    if &sig != b"SS+m" {
        return Err(ParseError::InvalidSignature(
            format!("Attendu 'SS+m', trouvé '{}'", String::from_utf8_lossy(&sig))
        ));
    }

    let version = cursor.read_u16::<LittleEndian>()?;

    match version {
        1 => parse_sspm_v1(&mut cursor),
        2 => parse_sspm_v2(&mut cursor),
        v => Err(ParseError::UnsupportedVersion(v)),
    }
}

/// SSPM v1 — sspmV1() L300-388 dans MapParser.cs
fn parse_sspm_v1(cursor: &mut Cursor<&[u8]>) -> Result<MapData, ParseError> {
    // Skip 2 reserved bytes (L306)
    cursor.read_u16::<LittleEndian>()?;

    // ID de la map (ligne terminée par \n)
    let id = read_line(cursor)?;

    // Nom "Artiste - Titre" ou juste "Titre" (L309-322)
    let name_line = read_line(cursor)?;
    let (artist, title) = if let Some(idx) = name_line.find(" - ") {
        (name_line[..idx].trim().to_string(), name_line[idx+3..].trim().to_string())
    } else {
        (String::new(), name_line.trim().to_string())
    };

    // Mappers séparés par '&' ou ',' (L324)
    let mappers_line = read_line(cursor)?;
    let mappers: Vec<String> = mappers_line
        .split([' ', '&', ','])
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    let map_length = cursor.read_u32::<LittleEndian>()?;
    let note_count = cursor.read_u32::<LittleEndian>()?;
    let difficulty = cursor.read_u8()?;

    // Cover (optionnel, flag = 2)
    let has_cover = cursor.read_u8()? == 2;
    let cover = if has_cover {
        let len = cursor.read_u64::<LittleEndian>()? as usize;
        let mut buf = vec![0u8; len];
        cursor.read_exact(&mut buf)?;
        buf
    } else { Vec::new() };

    // Audio (optionnel)
    let has_audio = cursor.read_u8()? != 0;
    let audio = if has_audio {
        let len = cursor.read_u64::<LittleEndian>()? as usize;
        let mut buf = vec![0u8; len];
        cursor.read_exact(&mut buf)?;
        buf
    } else { Vec::new() };

    // Notes — format identique à PHXMO (L347-370)
    // ATTENTION: dans SSPM v1, la transformation est x - 1, -y + 1 (L369)
    // notes[i] = new Note(i, millisecond, x - 1, -y + 1)
    let notes = parse_sspm_notes(cursor, note_count as usize, false, true)?;

    let audio_ext = MapData::detect_audio_ext(&audio).to_string();

    Ok(MapData {
        id,
        title,
        artist,
        mappers,
        difficulty,
        difficulty_name: String::new(),
        length_ms: map_length,
        audio,
        cover,
        audio_ext,
        notes,
    })
}

/// SSPM v2 — sspmV2() L391-543 dans MapParser.cs
fn parse_sspm_v2(cursor: &mut Cursor<&[u8]>) -> Result<MapData, ParseError> {
    cursor.seek(SeekFrom::Current(4))?;   // reserved (L397)
    cursor.seek(SeekFrom::Current(20))?;  // hash (L398)

    let map_length = cursor.read_u32::<LittleEndian>()?;
    let note_count = cursor.read_u32::<LittleEndian>()?;

    cursor.seek(SeekFrom::Current(4))?;   // marker count (L403)

    let difficulty = cursor.read_u8()?;

    cursor.seek(SeekFrom::Current(2))?;   // map rating (L407)

    let has_audio = cursor.read_u8()? != 0;
    let has_cover = cursor.read_u8()? != 0;

    cursor.seek(SeekFrom::Current(1))?;   // 1mod (L412)

    let custom_data_offset = cursor.read_u64::<LittleEndian>()?;
    let _custom_data_length = cursor.read_u64::<LittleEndian>()?;

    let audio_byte_offset = cursor.read_u64::<LittleEndian>()?;
    let audio_byte_length = cursor.read_u64::<LittleEndian>()?;

    let cover_byte_offset = cursor.read_u64::<LittleEndian>()?;
    let cover_byte_length = cursor.read_u64::<LittleEndian>()?;

    cursor.seek(SeekFrom::Current(16))?;  // marker defs offset & length (L423)

    let marker_byte_offset = cursor.read_u64::<LittleEndian>()?;
    cursor.seek(SeekFrom::Current(8))?;   // marker byte length (L427)

    // ID et Nom de la map
    let id_len = cursor.read_u16::<LittleEndian>()? as usize;
    let id = read_string(cursor, id_len)?;

    let name_len = cursor.read_u16::<LittleEndian>()? as usize;
    let name_str = read_string(cursor, name_len)?;
    let (artist, title) = if let Some(idx) = name_str.find(" - ") {
        (name_str[..idx].trim().to_string(), name_str[idx+3..].trim().to_string())
    } else {
        (String::new(), name_str.trim().to_string())
    };

    let song_name_len = cursor.read_u16::<LittleEndian>()? as usize;
    cursor.seek(SeekFrom::Current(song_name_len as i64))?; // skip song name (L450)

    let mapper_count = cursor.read_u16::<LittleEndian>()? as usize;
    let mut mappers = Vec::with_capacity(mapper_count);
    for _ in 0..mapper_count {
        let len = cursor.read_u16::<LittleEndian>()? as usize;
        mappers.push(read_string(cursor, len)?);
    }

    // Lire le difficulty_name depuis les custom data
    let mut difficulty_name = String::new();
    cursor.seek(SeekFrom::Start(custom_data_offset))?;
    cursor.seek(SeekFrom::Current(2))?;  // skip field count
    let key_len = cursor.read_u16::<LittleEndian>()? as usize;
    let key = read_string(cursor, key_len)?;
    if key == "difficulty_name" {
        let type_tag = cursor.read_u8()?;
        let val_len = match type_tag {
            9  => cursor.read_u16::<LittleEndian>()? as usize,
            11 => cursor.read_u32::<LittleEndian>()? as usize,
            _  => 0,
        };
        if val_len > 0 {
            difficulty_name = read_string(cursor, val_len)?;
        }
    }

    // Lire l'audio
    let audio = if has_audio {
        cursor.seek(SeekFrom::Start(audio_byte_offset))?;
        let mut buf = vec![0u8; audio_byte_length as usize];
        cursor.read_exact(&mut buf)?;
        buf
    } else { Vec::new() };

    // Lire la cover
    let cover = if has_cover {
        cursor.seek(SeekFrom::Start(cover_byte_offset))?;
        let mut buf = vec![0u8; cover_byte_length as usize];
        cursor.read_exact(&mut buf)?;
        buf
    } else { Vec::new() };

    // Lire les notes
    cursor.seek(SeekFrom::Start(marker_byte_offset))?;
    let notes = parse_sspm_v2_notes(cursor, note_count as usize)?;

    let audio_ext = MapData::detect_audio_ext(&audio).to_string();

    Ok(MapData {
        id,
        title,
        artist,
        mappers,
        difficulty,
        difficulty_name,
        length_ms: map_length,
        audio,
        cover,
        audio_ext,
        notes,
    })
}

/// Parser les notes SSPM v1 — même format que PHXMO mais avec
/// une transformation Y inversée : note = (x-1, -y+1)
fn parse_sspm_notes(
    cursor: &mut Cursor<&[u8]>,
    count: usize,
    skip_marker_type: bool,  // SSPM v2 a un octet "marker type" supplémentaire
    invert_y: bool,          // SSPM v1: y = -rawY + 1
) -> Result<Vec<NoteData>, ParseError> {
    let mut notes = Vec::with_capacity(count);

    for i in 0..count {
        let ms = cursor.read_u32::<LittleEndian>()?;

        if skip_marker_type {
            cursor.read_u8()?; // marker type (toujours note) — L506
        }

        let quantum = cursor.read_u8()? != 0;

        let (x, y) = if quantum {
            let x = cursor.read_f32::<LittleEndian>()?;
            let y = cursor.read_f32::<LittleEndian>()?;
            // SSPM v1 L369: notes[i] = new Note(i, ms, x - 1, -y + 1)
            let y_out = if invert_y { -y + 1.0 } else { y - 1.0 };
            (x - 1.0, y_out)
        } else {
            let raw_x = cursor.read_u8()? as f32;
            let raw_y = cursor.read_u8()? as f32;
            let y_out = if invert_y { -raw_y + 1.0 } else { raw_y - 1.0 };
            (raw_x - 1.0, y_out)
        };

        notes.push(NoteData { index: i, millisecond: ms, x, y });
    }

    notes.sort_by_key(|n| n.millisecond);
    for (i, note) in notes.iter_mut().enumerate() {
        note.index = i;
    }

    Ok(notes)
}

/// Parser les notes SSPM v2 — ajoute le skip du marker type (L506)
fn parse_sspm_v2_notes(cursor: &mut Cursor<&[u8]>, count: usize) -> Result<Vec<NoteData>, ParseError> {
    let mut notes = Vec::with_capacity(count);

    for i in 0..count {
        let ms = cursor.read_u32::<LittleEndian>()?;
        cursor.read_u8()?;  // marker type — toujours "note" (L506)

        let quantum = cursor.read_u8()? != 0;

        let (x, y) = if quantum {
            let x = cursor.read_f32::<LittleEndian>()?;
            let y = cursor.read_f32::<LittleEndian>()?;
            // SSPM v2 L523: notes[i] = new Note(0, ms, x - 1, -y + 1)
            (x - 1.0, -y + 1.0)
        } else {
            let raw_x = cursor.read_u8()? as f32;
            let raw_y = cursor.read_u8()? as f32;
            (raw_x - 1.0, -raw_y + 1.0)
        };

        notes.push(NoteData { index: i, millisecond: ms, x, y });
    }

    notes.sort_by_key(|n| n.millisecond);
    for (i, note) in notes.iter_mut().enumerate() {
        note.index = i;
    }

    Ok(notes)
}

// ==============================================================================
// FORMAT SSMapV1 (.txt) — MapParser.SSMapV1()
// Format CSV legacy: "ms|x|y,ms|x|y,..." (une seule ligne)
// Traduction de SSMapV1() L183-223
// ==============================================================================

pub fn parse_ssmap_v1_bytes(bytes: &[u8]) -> Result<MapData, ParseError> {
    let content = String::from_utf8_lossy(bytes);
    let line = content.lines().next().unwrap_or("");

    let parts: Vec<&str> = line.split(',').collect();
    // parts[0] = éventuellement un header, parts[1..] = notes
    let mut notes = Vec::new();

    for (i, part) in parts.iter().enumerate().skip(1) {
        let sub: Vec<&str> = part.split('|').collect();
        if sub.len() < 3 { continue; }

        // SSMapV1 L199: notes[i-1] = new Note(i-1, subsplit[2].ToInt(), -subsplit[0].ToFloat() + 1, subsplit[1].ToFloat() - 1)
        let raw_x: f32 = sub[0].trim().parse().unwrap_or(0.0);
        let raw_y: f32 = sub[1].trim().parse().unwrap_or(0.0);
        let ms: u32    = sub[2].trim().parse().unwrap_or(0);

        notes.push(NoteData {
            index: i - 1,
            millisecond: ms,
            x: -raw_x + 1.0,  // -subsplit[0] + 1
            y: raw_y - 1.0,   // subsplit[1] - 1
        });
    }

    notes.sort_by_key(|n| n.millisecond);
    for (i, note) in notes.iter_mut().enumerate() {
        note.index = i;
    }

    let length_ms = notes.last().map(|n| n.millisecond).unwrap_or(0);

    Ok(MapData {
        id: String::from("txt_map"),
        title: String::from("Unknown"),
        artist: String::new(),
        mappers: vec![String::from("N/A")],
        difficulty: 0,
        difficulty_name: String::from("N/A"),
        length_ms,
        audio: Vec::new(),
        cover: Vec::new(),
        audio_ext: String::from("mp3"),
        notes,
    })
}

// ==============================================================================
// HELPERS
// ==============================================================================

/// Lire une chaîne terminée par '\n' depuis un cursor
fn read_line(cursor: &mut Cursor<&[u8]>) -> Result<String, ParseError> {
    let mut s = Vec::new();
    loop {
        let b = cursor.read_u8()?;
        if b == b'\n' || b == 0 { break; }
        s.push(b);
    }
    Ok(String::from_utf8_lossy(&s).trim_end_matches('\r').to_string())
}

/// Lire une chaîne de longueur fixe depuis un cursor
fn read_string(cursor: &mut Cursor<&[u8]>, len: usize) -> Result<String, ParseError> {
    let mut buf = vec![0u8; len];
    cursor.read_exact(&mut buf)?;
    Ok(String::from_utf8_lossy(&buf).to_string())
}

// ==============================================================================
// TESTS
// ==============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ssmap_v1_parsing() {
        // Format: "header,x|y|ms,x|y|ms"
        let data = b"header,0|0|1000,-1|1|2000,1|-1|3000";
        let map = parse_ssmap_v1_bytes(data).unwrap();
        assert_eq!(map.notes.len(), 3);
        // Note 1: raw_x=0, raw_y=0, ms=1000 → x=-0+1=1.0, y=0-1=-1.0
        assert_eq!(map.notes[0].millisecond, 1000);
        assert_eq!(map.notes[0].x, 1.0);
        assert_eq!(map.notes[0].y, -1.0);
    }

    #[test]
    fn test_phxmo_binary_discrete() {
        // Simuler 1 note discrète non-quantum: ms=5000, x=1 (→0), y=2 (→1)
        let mut data = Vec::new();
        data.extend_from_slice(&12u32.to_le_bytes());  // type_count
        data.extend_from_slice(&1u32.to_le_bytes());   // note_count
        data.extend_from_slice(&5000u32.to_le_bytes()); // ms
        data.push(0); // not quantum
        data.push(1); // x byte: 1 - 1 = 0
        data.push(2); // y byte: 2 - 1 = 1

        let notes = parse_phxmo_binary(&data).unwrap();
        assert_eq!(notes.len(), 1);
        assert_eq!(notes[0].millisecond, 5000);
        assert_eq!(notes[0].x, 0.0);
        assert_eq!(notes[0].y, 1.0);
    }
}
