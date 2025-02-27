use std::collections::HashMap;
use std::{fs, io};
use std::path::{Path, PathBuf};

pub fn collect_files(paths: &mut Vec<PathBuf>, dir: &Path) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_dir() {
            collect_files(paths, &path)?;
        } else {
            paths.push(entry.path());
        }
    }
    Ok(())
}
pub fn get_current_pak_characteristics(mod_contents: Vec<String>) -> String {
    let character_map: HashMap<&str, &str> = [
        ("1011", "Hulk"),
        ("1014", "Punisher"),
        ("1015", "Storm"),
        ("1016", "Loki"),
        ("1018", "Dr.Strange"),
        ("1020", "Mantis"),
        ("1021", "Hawkeye"),
        ("1022", "Captain America"),
        ("1023", "Raccoon"),
        ("1024", "Hela"),
        ("1025", "CND"),
        ("1026", "Black Panther"),
        ("1027", "Groot"),
        ("1029", "Magik"),
        ("1030", "Moonknight"),
        ("1031", "Luna Snow"),
        ("1032", "Squirrel Girl"),
        ("1033", "Black Widow"),
        ("1034", "Iron Man"),
        ("1035", "Venom"),
        ("1036", "Spider Man"),
        ("1037", "Magneto"),
        ("1038", "Scarlet Witch"),
        ("1039", "Thor"),
        ("1040", "Mr Fantastic"),
        ("1041", "Winter Soldier"),
        ("1042", "Peni Parker"),
        ("1043", "Starlord"),
        ("1045", "Namor"),
        ("1046", "Adam Warlock"),
        ("1047", "Jeff"),
        ("1048", "Psylocke"),
        ("1049", "Wolverine"),
        ("1050", "Invisible Woman"),
        ("1052", "Iron Fist"),
        ("4017", "Announcer (Galacta)"),
        ("8021", "Loki's extra yapping"),
        ("8031", "Random NPCs"),
        ("8032", "Random NPCs"),
        ("8041", "Random NPCs"),
        ("8042", "Random NPCs"),
        ("8043", "Random NPCs"),
        ("8063", "Male NPC"),
    ]
        .iter()
        .cloned()
        .collect();

    for file in &mod_contents {
        if let Some(stripped) = file.strip_prefix("Marvel/Content/Marvel/") {
            let category = stripped.split('/').into_iter().next().unwrap_or_default();

            if category == "Characters" {
                // Extract the ID from the file path
                let parts: Vec<&str> = stripped.split('/').collect();
                if parts.len() > 1 {
                    let id = parts[1]; // Assuming ID is in second position
                    if let Some(character_name) = character_map.get(id) {
                        return format!("Character ({})", character_name);
                    }
                }
                return "Character (Unknown)".to_string();
            } else if category == "UI" {
                return "UI".to_string();
            }
        }
    }
    "Unknown".to_string()
}

