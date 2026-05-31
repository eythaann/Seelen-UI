#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SystemLanguage {
    pub id: String,
    pub code: String,
    pub name: String,
    pub native_name: String,
    /// List of loaded keyboard layouts for this language
    pub keyboard_layouts: Vec<KeyboardLayout>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
pub struct KeyboardLayout {
    /// HKL-based: KLID e.g. "00000409". TSF: full tip e.g. "0412:{CLSID}{ProfileGUID}".
    pub id: String,
    /// Full input method tip: "LANGID:KLID" or "LANGID:{CLSID}{ProfileGUID}".
    /// Passed back to the backend as-is when activating this layout.
    pub handle: String,
    pub display_name: String,
    pub active: bool,
}

/// Current IME conversion state.
///
/// These flags originate from IMM32 (`IME_CMODE_*`) and are shared between
/// Chinese, Japanese and Korean IMEs. Not every language or IME uses every
/// flag. Modern TSF-based IMEs may also ignore some of them.
///
/// # Language support
///
/// | Field | Chinese | Japanese | Korean |
/// |---------|---------|----------|---------|
/// | `native` | ✅ | ✅ | ✅ |
/// | `full_shape` | ✅ | ✅ | ⚠️ Rare |
/// | `katakana` | ❌ | ✅ | ❌ |
/// | `roman` | ❌ | ✅ | ❌ |
/// | `hanja` | ❌ | ❌ | ✅ |
/// | `open` | ✅ | ✅ | ✅ |
///
/// # Common states
///
/// ## Japanese
///
/// | State | open | native | katakana | full_shape |
/// |---------|---------|---------|---------|---------|
/// | A (Direct Input) | false | false | false | false |
/// | A (Half-width Alphanumeric) | true | false | false | false |
/// | Ａ (Full-width Alphanumeric) | true | false | false | true |
/// | あ (Hiragana) | true | true | false | false |
/// | ア (Full-width Katakana) | true | true | true | true |
/// | ｱ (Half-width Katakana) | true | true | true | false |
///
/// ## Chinese (Pinyin)
///
/// | State | open | native | full_shape |
/// |---------|---------|---------|---------|
/// | English | false* | false | false |
/// | Chinese | true | true | false |
/// | Chinese Full-width | true | true | true |
///
/// *Some IMEs keep `open=true` even while in English mode.
///
/// ## Korean
///
/// | State | open | native | hanja |
/// |---------|---------|---------|---------|
/// | English | false | false | false |
/// | Hangul | true | true | false |
/// | Hanja Conversion | true | true | true |
///
/// # Notes
///
/// - `native` corresponds to `IME_CMODE_NATIVE` (`0x0001`).
///   The meaning depends on the active language:
///   - Chinese: Chinese input mode.
///   - Japanese: Kana/Hiragana input mode.
///   - Korean: Hangul input mode.
///
/// - `full_shape` corresponds to `IME_CMODE_FULLSHAPE` (`0x0008`).
///   Enables full-width characters.
///   Examples:
///   - `A` → `Ａ`
///   - `1` → `１`
///   - `ｱ` ↔ `ア`
///
/// - `katakana` corresponds to `IME_CMODE_KATAKANA` (`0x0002`).
///   Japanese-only.
///   When `native=true`:
///   - `false` = Hiragana (`あ`)
///   - `true` = Katakana (`ア` or `ｱ`)
///
/// - `roman` corresponds to `IME_CMODE_ROMAN` (`0x0010`).
///   Japanese-only.
///   Indicates Romaji input mode:
///   - `ka` → `か`
///   - `shi` → `し`
///
///   When disabled, the IME may use direct Kana input from a Japanese keyboard.
///   Most modern users keep this enabled at all times.
///
/// - `hanja` corresponds to `IME_CMODE_HANJACONVERT` (`0x0040`).
///   Korean-only.
///   Indicates Hanja conversion mode.
///
/// - `open` corresponds to `ImmGetOpenStatus()` or
///   `IMC_GETOPENSTATUS`.
///   Indicates whether the IME is active.
///
/// # Unsupported IMM32 flags
///
/// The following IMM32 flags exist but are not currently exposed because
/// they are uncommon, legacy, or rarely implemented by modern IMEs:
///
/// - `IME_CMODE_CHARCODE`
/// - `IME_CMODE_EUDC`
/// - `IME_CMODE_FIXED`
/// - `IME_CMODE_NATIVESYMBOL`
/// - `IME_CMODE_SOFTKBD`
/// - `IME_CMODE_NOCONVERSION`
/// - `IME_CMODE_SYMBOL`
///
/// Modern TSF-based IMEs often do not report these consistently.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct ImeState {
    /// Native language mode.
    ///
    /// Meaning depends on the active language:
    /// - Chinese: Chinese mode.
    /// - Japanese: Hiragana/Kana mode.
    /// - Korean: Hangul mode.
    pub native: bool,

    /// Full-width character mode.
    ///
    /// Examples:
    /// - A -> Ａ
    /// - 1 -> １
    /// - ｱ -> ア
    pub full_shape: bool,

    /// Japanese-only.
    ///
    /// When `native` is enabled:
    /// - false = Hiragana
    /// - true = Katakana
    pub katakana: bool,

    /// Japanese-only.
    ///
    /// Romaji input mode:
    /// - ka -> か
    /// - shi -> し
    ///
    /// Usually enabled on modern systems.
    pub roman: bool,

    /// Korean-only.
    ///
    /// Hanja conversion mode.
    pub hanja: bool,

    /// Whether the IME is currently active/open.
    pub open: bool,
}
