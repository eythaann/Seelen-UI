use std::path::PathBuf;

use owo_colors::OwoColorize;
use seelen_core::{
    constants::SUPPORTED_LANGUAGES,
    resource::{ResourceText, SluResource},
    state::{IconPack, Plugin, Theme, Wallpaper, Widget},
};
use slu_ipc::commands::{ClapResourceKind, ResourceManagerCli, ResourceSubCommand};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

pub async fn process(cmd: ResourceManagerCli) -> Result<()> {
    match cmd.subcommand {
        ResourceSubCommand::Bundle { kind, path } => bundle(kind, path)?,
        ResourceSubCommand::Translate { path, source_lang } => translate(path, source_lang).await?,
        _ => return Err("This command needs Seelen UI to be running".into()),
    }
    Ok(())
}

fn bundle(kind: ClapResourceKind, path: PathBuf) -> Result<()> {
    let mut to_store_path = path.clone();

    let format = time::macros::format_description!("[year]-[month]-[day] [hour]-[minute]-[second]");
    let date = time::OffsetDateTime::now_local().map_err(time::Error::IndeterminateOffset)?;
    let date_str = date.format(&format).map_err(time::Error::Format)?;
    let filename = format!("bundle {date_str}.yml");

    if to_store_path.is_dir() {
        to_store_path.push(filename);
    } else {
        to_store_path.set_file_name(filename);
    }

    let e = |e: seelen_core::error::SeelenLibError| -> Box<dyn std::error::Error + Send + Sync> {
        e.to_string().into()
    };

    match kind {
        ClapResourceKind::Theme => {
            let mut theme = Theme::load_ext(&path, false).map_err(e)?;
            theme.metadata.internal.path = to_store_path.clone();
            theme.save().map_err(e)?;
        }
        ClapResourceKind::Plugin => {
            let mut plugin = Plugin::load_ext(&path, false).map_err(e)?;
            plugin.metadata.internal.path = to_store_path.clone();
            plugin.save().map_err(e)?;
        }
        ClapResourceKind::Widget => {
            let mut widget = Widget::load_ext(&path, false).map_err(e)?;
            widget.metadata.internal.path = to_store_path.clone();
            widget.save().map_err(e)?;
        }
        ClapResourceKind::IconPack => {
            let mut icon_pack = IconPack::load_ext(&path, false).map_err(e)?;
            icon_pack.metadata.internal.path = to_store_path.clone();
            icon_pack.save().map_err(e)?;
        }
        ClapResourceKind::Wallpaper => {
            let mut wallpaper = Wallpaper::load_ext(&path, false).map_err(e)?;
            wallpaper.metadata.internal.path = to_store_path.clone();
            wallpaper.save().map_err(e)?;
        }
        _ => return Err("Not implemented".into()),
    }

    println!(
        "Bundle created successfully at: {}",
        to_store_path.display()
    );
    Ok(())
}

async fn translate(path: PathBuf, source_lang: Option<String>) -> Result<()> {
    let file = std::fs::File::open(&path)?;
    let mut texts: ResourceText = serde_yaml::from_reader(file)?;

    let code = source_lang.unwrap_or_else(|| "en".to_string());

    if !texts.has(&code) {
        return Err(format!("Source Language ({code}) not found.").into());
    }

    let source = texts.get(&code).to_owned();
    let total = SUPPORTED_LANGUAGES.len();

    let longest_lang = SUPPORTED_LANGUAGES
        .iter()
        .map(|lang| lang.en_label.len())
        .max()
        .unwrap_or(0);

    for (idx, lang) in SUPPORTED_LANGUAGES.iter().enumerate() {
        let step = if idx < 9 {
            format!("0{}", idx + 1)
        } else {
            (idx + 1).to_string()
        };

        let label = format!(
            "{}{}",
            lang.en_label,
            " ".repeat(longest_lang - lang.en_label.len())
        );

        if texts.has(lang.value) {
            println!(
                "[{step}/{total}] {} => {}",
                label.bright_black(),
                "Skipped".bright_black()
            );
            continue;
        }

        match translate_text(&source, &code, lang.value).await {
            Ok(value) => {
                println!(
                    "[{step}/{total}] {} => \"{}\"",
                    label.bold().bright_green(),
                    value
                );
                texts.set(lang.value.to_string(), value);
            }
            Err(err) => {
                eprintln!(
                    "[{step}/{total}] {} => Error translating to {} ({}): {}",
                    label.bold().bright_red(),
                    lang.en_label,
                    lang.value,
                    err
                );
            }
        }
    }

    let file = std::fs::File::create(&path)?;
    serde_yaml::to_writer(file, &texts)?;
    Ok(())
}

async fn translate_text(source: &str, source_lang: &str, mut target_lang: &str) -> Result<String> {
    use translators::{GoogleTranslator, Translator};
    let translator = GoogleTranslator::default();

    if target_lang == "zh" {
        target_lang = "zh-CN";
    }
    if target_lang == "pt" {
        target_lang = "pt-BR";
    }

    let translated = translator
        .translate_async(source, source_lang, target_lang)
        .await?;
    Ok(translated)
}
