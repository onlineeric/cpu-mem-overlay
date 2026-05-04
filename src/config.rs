//! Overlay configuration: TOML loading + per-field validators.
//!
//! Resolves `<exe-dir>/cpu-mem-overlay.toml` once at startup. Missing file,
//! unparseable file, unknown keys, and per-field invalid values all silently
//! fall back to documented defaults (FR-002 / FR-003 / FR-004 / FR-005 /
//! FR-006a). Each field has its own validator so one bad value can never
//! reject other valid siblings.

use std::path::Path;
use std::time::Duration;

use serde::Deserialize;

use crate::color::parse_hex_color;

const CONFIG_FILENAME: &str = "cpu-mem-overlay.toml";

const DEFAULT_REFRESH_INTERVAL_MS: u64 = 1000;
const MIN_REFRESH_INTERVAL_MS: u64 = 100;
const DEFAULT_BACKGROUND_COLOR: [u8; 4] = [0, 0, 0, 0];
const DEFAULT_FONT_SIZE: f32 = 12.0;
const MAX_FONT_SIZE: f32 = 256.0;
const DEFAULT_DRAGGABLE: bool = false;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Anchor {
    Default,
    VirtualScreen { x: i32, y: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct OverlayConfig {
    pub(crate) refresh_interval: Duration,
    pub(crate) anchor: Anchor,
    pub(crate) background_color: [u8; 4],
    pub(crate) font_size: f32,
    pub(crate) draggable: bool,
}

impl Default for OverlayConfig {
    fn default() -> Self {
        Self {
            refresh_interval: Duration::from_millis(DEFAULT_REFRESH_INTERVAL_MS),
            anchor: Anchor::Default,
            background_color: DEFAULT_BACKGROUND_COLOR,
            font_size: DEFAULT_FONT_SIZE,
            draggable: DEFAULT_DRAGGABLE,
        }
    }
}

#[derive(Debug, Default, Deserialize)]
struct RawOverlayConfig {
    refresh_interval_ms: Option<u64>,
    anchor_position: Option<[i32; 2]>,
    background_color: Option<String>,
    font_size: Option<f32>,
    draggable: Option<bool>,
}

pub(crate) fn load_from_exe_dir() -> OverlayConfig {
    let Ok(exe) = std::env::current_exe() else {
        return OverlayConfig::default();
    };
    let Some(dir) = exe.parent() else {
        return OverlayConfig::default();
    };
    load_from_dir(dir)
}

pub(crate) fn load_from_dir(dir: &Path) -> OverlayConfig {
    let path = dir.join(CONFIG_FILENAME);
    let Ok(contents) = std::fs::read_to_string(&path) else {
        return OverlayConfig::default();
    };
    let Ok(raw) = toml::from_str::<RawOverlayConfig>(&contents) else {
        return OverlayConfig::default();
    };
    from_raw(raw)
}

fn from_raw(raw: RawOverlayConfig) -> OverlayConfig {
    OverlayConfig {
        refresh_interval: validate_refresh_interval_ms(raw.refresh_interval_ms),
        anchor: validate_anchor(raw.anchor_position),
        background_color: validate_background_color(raw.background_color),
        font_size: validate_font_size(raw.font_size),
        draggable: validate_draggable(raw.draggable),
    }
}

fn validate_refresh_interval_ms(raw: Option<u64>) -> Duration {
    match raw {
        Some(ms) if ms >= MIN_REFRESH_INTERVAL_MS => Duration::from_millis(ms),
        _ => Duration::from_millis(DEFAULT_REFRESH_INTERVAL_MS),
    }
}

fn validate_anchor(raw: Option<[i32; 2]>) -> Anchor {
    match raw {
        Some([x, y]) => Anchor::VirtualScreen { x, y },
        None => Anchor::Default,
    }
}

fn validate_background_color(raw: Option<String>) -> [u8; 4] {
    match raw {
        Some(s) => parse_hex_color(&s).unwrap_or(DEFAULT_BACKGROUND_COLOR),
        None => DEFAULT_BACKGROUND_COLOR,
    }
}

fn validate_font_size(raw: Option<f32>) -> f32 {
    match raw {
        Some(f) if f.is_finite() && f > 0.0 && f <= MAX_FONT_SIZE => f,
        _ => DEFAULT_FONT_SIZE,
    }
}

fn validate_draggable(raw: Option<bool>) -> bool {
    raw.unwrap_or(DEFAULT_DRAGGABLE)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;

    fn parse(toml_str: &str) -> OverlayConfig {
        let raw: RawOverlayConfig = toml::from_str(toml_str).expect("test TOML must parse");
        from_raw(raw)
    }

    #[test]
    fn missing_file_returns_defaults() {
        let dir = std::env::temp_dir().join(format!("cmo-test-missing-{}", uniq()));
        fs::create_dir_all(&dir).unwrap();
        let cfg = load_from_dir(&dir);
        assert_eq!(cfg, OverlayConfig::default());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn unparseable_toml_returns_defaults() {
        let dir = std::env::temp_dir().join(format!("cmo-test-bad-{}", uniq()));
        fs::create_dir_all(&dir).unwrap();
        let path = dir.join("cpu-mem-overlay.toml");
        let mut f = fs::File::create(&path).unwrap();
        writeln!(f, "this is not toml = = =").unwrap();
        let cfg = load_from_dir(&dir);
        assert_eq!(cfg, OverlayConfig::default());
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn partial_config_applies_set_keys_only() {
        let cfg = parse("refresh_interval_ms = 250\n");
        assert_eq!(cfg.refresh_interval, Duration::from_millis(250));
        assert_eq!(cfg.anchor, Anchor::Default);
        assert_eq!(cfg.background_color, DEFAULT_BACKGROUND_COLOR);
        assert_eq!(cfg.font_size, DEFAULT_FONT_SIZE);
        assert!(!cfg.draggable);
    }

    #[test]
    fn refresh_interval_ms_at_minimum_accepted() {
        let cfg = parse("refresh_interval_ms = 100\n");
        assert_eq!(cfg.refresh_interval, Duration::from_millis(100));
    }

    #[test]
    fn refresh_interval_ms_below_minimum_falls_back() {
        let cfg = parse("refresh_interval_ms = 99\n");
        assert_eq!(
            cfg.refresh_interval,
            Duration::from_millis(DEFAULT_REFRESH_INTERVAL_MS)
        );
    }

    #[test]
    fn refresh_interval_ms_zero_falls_back() {
        let cfg = parse("refresh_interval_ms = 0\n");
        assert_eq!(
            cfg.refresh_interval,
            Duration::from_millis(DEFAULT_REFRESH_INTERVAL_MS)
        );
    }

    #[test]
    fn font_size_zero_falls_back() {
        let cfg = parse("font_size = 0.0\n");
        assert_eq!(cfg.font_size, DEFAULT_FONT_SIZE);
    }

    #[test]
    fn font_size_negative_falls_back() {
        let cfg = parse("font_size = -5.0\n");
        assert_eq!(cfg.font_size, DEFAULT_FONT_SIZE);
    }

    #[test]
    fn font_size_nan_falls_back() {
        // NaN can't be expressed in TOML, so go through the validator directly.
        assert_eq!(validate_font_size(Some(f32::NAN)), DEFAULT_FONT_SIZE);
        assert_eq!(validate_font_size(Some(f32::INFINITY)), DEFAULT_FONT_SIZE);
        assert_eq!(
            validate_font_size(Some(f32::NEG_INFINITY)),
            DEFAULT_FONT_SIZE
        );
    }

    #[test]
    fn font_size_default_value_accepted() {
        let cfg = parse("font_size = 14.0\n");
        assert_eq!(cfg.font_size, 14.0);
    }

    #[test]
    fn font_size_above_cap_falls_back() {
        let cfg = parse("font_size = 257.0\n");
        assert_eq!(cfg.font_size, DEFAULT_FONT_SIZE);
    }

    #[test]
    fn anchor_position_set() {
        let cfg = parse("anchor_position = [10, 20]\n");
        assert_eq!(cfg.anchor, Anchor::VirtualScreen { x: 10, y: 20 });
    }

    #[test]
    fn anchor_position_negative_supported() {
        let cfg = parse("anchor_position = [-100, -50]\n");
        assert_eq!(cfg.anchor, Anchor::VirtualScreen { x: -100, y: -50 });
    }

    #[test]
    fn anchor_position_missing_uses_default() {
        let cfg = parse("");
        assert_eq!(cfg.anchor, Anchor::Default);
    }

    #[test]
    fn malformed_background_color_falls_back() {
        let cfg = parse("background_color = \"not a color\"\n");
        assert_eq!(cfg.background_color, DEFAULT_BACKGROUND_COLOR);
    }

    #[test]
    fn fully_transparent_background_accepted() {
        let cfg = parse("background_color = \"#00000000\"\n");
        assert_eq!(cfg.background_color, [0, 0, 0, 0]);
    }

    #[test]
    fn six_digit_background_color_accepted_as_opaque() {
        let cfg = parse("background_color = \"#101820\"\n");
        assert_eq!(cfg.background_color, [0x10, 0x18, 0x20, 0xFF]);
    }

    #[test]
    fn unknown_key_silently_ignored() {
        let cfg = parse("refresh_interval_ms = 500\nunrecognized_key = \"anything\"\n");
        assert_eq!(cfg.refresh_interval, Duration::from_millis(500));
    }

    #[test]
    fn single_invalid_key_only_that_field_falls_back() {
        let cfg = parse(concat!(
            "refresh_interval_ms = 500\n",
            "background_color = \"not a color\"\n",
            "font_size = 18.0\n",
        ));
        assert_eq!(cfg.refresh_interval, Duration::from_millis(500));
        assert_eq!(cfg.background_color, DEFAULT_BACKGROUND_COLOR);
        assert_eq!(cfg.font_size, 18.0);
    }

    #[test]
    fn draggable_true_accepted() {
        let cfg = parse("draggable = true\n");
        assert!(cfg.draggable);
    }

    #[test]
    fn draggable_default_false() {
        let cfg = parse("");
        assert!(!cfg.draggable);
    }

    #[test]
    fn full_config_round_trip() {
        let cfg = parse(concat!(
            "refresh_interval_ms = 250\n",
            "anchor_position = [200, 100]\n",
            "background_color = \"#FFFFFFFF\"\n",
            "font_size = 22.0\n",
            "draggable = true\n",
        ));
        assert_eq!(cfg.refresh_interval, Duration::from_millis(250));
        assert_eq!(cfg.anchor, Anchor::VirtualScreen { x: 200, y: 100 });
        assert_eq!(cfg.background_color, [0xFF, 0xFF, 0xFF, 0xFF]);
        assert_eq!(cfg.font_size, 22.0);
        assert!(cfg.draggable);
    }

    fn uniq() -> u128 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    }
}
