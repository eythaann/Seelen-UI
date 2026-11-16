/// A single modified virtual-key chord. Never accept input scripts or broadcast media keys.
pub fn parse_media_shortcut(value: &str) -> Result<Vec<u16>, &'static str> {
    let parts: Vec<_> = value
        .split('+')
        .map(|part| part.trim().to_ascii_uppercase())
        .collect();
    if !(2..=4).contains(&parts.len()) {
        return Err("Use Ctrl, Alt or Shift followed by one key");
    }
    let mut keys = Vec::new();
    for modifier in &parts[..parts.len() - 1] {
        let key = match modifier.as_str() {
            "CTRL" | "CONTROL" => 0x11,
            "ALT" => 0x12,
            "SHIFT" => 0x10,
            _ => return Err("Unsupported shortcut modifier"),
        };
        if keys.contains(&key) {
            return Err("Duplicate shortcut modifier");
        }
        keys.push(key);
    }
    let main = parts.last().unwrap();
    let key = match main.as_str() {
        "-" => 0xBD,
        "=" => 0xBB,
        "SPACE" => 0x20,
        "LEFT" => 0x25,
        "UP" => 0x26,
        "RIGHT" => 0x27,
        "DOWN" => 0x28,
        _ if main.len() == 1 && main.as_bytes()[0].is_ascii_alphanumeric() => {
            main.as_bytes()[0] as u16
        }
        _ => {
            let number = main
                .strip_prefix('F')
                .and_then(|number| number.parse::<u16>().ok());
            match number {
                Some(number @ 1..=24) if main.len() <= 3 && !main.starts_with("F0") => {
                    0x70 + number - 1
                }
                _ => return Err("Unsupported shortcut key"),
            }
        }
    };
    keys.push(key);
    Ok(keys)
}

#[cfg(test)]
mod tests {
    use super::parse_media_shortcut;

    #[test]
    fn parses_player_chords_as_virtual_keys() {
        assert_eq!(parse_media_shortcut("Ctrl+0").unwrap(), vec![0x11, 0x30]);
        assert_eq!(parse_media_shortcut("Ctrl+-").unwrap(), vec![0x11, 0xBD]);
        assert_eq!(parse_media_shortcut("Ctrl+=").unwrap(), vec![0x11, 0xBB]);
        assert_eq!(
            parse_media_shortcut(" ctrl + alt + F12 ").unwrap(),
            vec![0x11, 0x12, 0x7B]
        );
    }

    #[test]
    fn rejects_unmodified_keys_scripts_and_broadcast_media_keys() {
        for chord in [
            "",
            "0",
            "Ctrl",
            "Ctrl+Ctrl+P",
            "Ctrl+P+Q",
            "Win+P",
            "Ctrl+F25",
            "Ctrl+F01",
            "Ctrl+F001",
            "Ctrl+MediaPlayPause",
            "Ctrl+{P}",
        ] {
            assert!(parse_media_shortcut(chord).is_err(), "accepted {chord:?}");
        }
    }
}
