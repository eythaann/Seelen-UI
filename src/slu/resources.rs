use std::{
    path::PathBuf,
    time::{Duration, Instant},
};

use owo_colors::OwoColorize;
use seelen_core::{
    constants::SUPPORTED_LANGUAGES,
    resource::{ResourceText, SluResource},
    state::{IconPack, Plugin, Theme, Wallpaper, Widget},
};
use slu_ipc::commands::{ClapResourceKind, ResourceManagerCli, ResourceSubCommand};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Minimum time between consecutive requests to google translate.
const MIN_REQUEST_INTERVAL: Duration = Duration::from_millis(500);
const RETRY_BASE_DELAY: Duration = Duration::from_secs(1);
const MAX_ATTEMPTS: u32 = 5;
const BATCH_RPC_ID: &str = "MkEWBc";

pub async fn process(cmd: ResourceManagerCli) -> Result<()> {
    match cmd.subcommand {
        ResourceSubCommand::Bundle { kind, path } => bundle(kind, path).await?,
        ResourceSubCommand::Translate { path, source_lang } => translate(path, source_lang).await?,
        _ => return Err("This command needs Seelen UI to be running".into()),
    }
    Ok(())
}

async fn bundle(kind: ClapResourceKind, path: PathBuf) -> Result<()> {
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
            let mut theme = Theme::load_ext(&path, false).await.map_err(e)?;
            theme.metadata.internal.path = to_store_path.clone();
            theme.save().await.map_err(e)?;
        }
        ClapResourceKind::Plugin => {
            let mut plugin = Plugin::load_ext(&path, false).await.map_err(e)?;
            plugin.metadata.internal.path = to_store_path.clone();
            plugin.save().await.map_err(e)?;
        }
        ClapResourceKind::Widget => {
            let mut widget = Widget::load_ext(&path, false).await.map_err(e)?;
            widget.metadata.internal.path = to_store_path.clone();
            widget.save().await.map_err(e)?;
        }
        ClapResourceKind::IconPack => {
            let mut icon_pack = IconPack::load_ext(&path, false).await.map_err(e)?;
            icon_pack.metadata.internal.path = to_store_path.clone();
            icon_pack.save().await.map_err(e)?;
        }
        ClapResourceKind::Wallpaper => {
            let mut wallpaper = Wallpaper::load_ext(&path, false).await.map_err(e)?;
            wallpaper.metadata.internal.path = to_store_path.clone();
            wallpaper.save().await.map_err(e)?;
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

    let mut translator = GoogleTranslator::new()?;
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

        match translator.translate(&source, &code, lang.value).await {
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

/// Google translate client, mirrors `google-translate-api-x` as used by @seelen/translation-toolkit:
/// tries the `single` endpoint and falls back to `batchexecute` (each one has its own rate limit),
/// throttling consecutive requests and retrying with backoff.
/// Note: the `/m` html endpoint is blocked by google.
struct GoogleTranslator {
    client: reqwest::Client,
    last_request: Option<Instant>,
}

impl GoogleTranslator {
    fn new() -> Result<Self> {
        // force IPv4, google rate limits IPv6 addresses more aggressively
        let client = reqwest::Client::builder()
            .local_address(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED))
            .build()?;
        Ok(Self {
            client,
            last_request: None,
        })
    }

    async fn translate(
        &mut self,
        source: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        let target_lang = match target_lang {
            "zh" => "zh-CN",
            "pt" => "pt-BR",
            other => other,
        };

        let mut last_err = String::new();
        for attempt in 0..MAX_ATTEMPTS {
            if attempt > 0 {
                // exponential backoff: 1s, 2s, 4s, 8s
                tokio::time::sleep(RETRY_BASE_DELAY * 2u32.pow(attempt - 1)).await;
            }

            self.throttle().await;
            let single_err = match self.single(source, source_lang, target_lang).await {
                Ok(text) => return Ok(text),
                Err(err) => err,
            };

            self.throttle().await;
            let batch_err = match self.batch(source, source_lang, target_lang).await {
                Ok(text) => return Ok(text),
                Err(err) => err,
            };

            last_err = format!("{single_err} | {batch_err}");
        }
        Err(format!("{last_err} (after {MAX_ATTEMPTS} attempts)").into())
    }

    /// Ensures a minimum delay between consecutive requests.
    async fn throttle(&mut self) {
        if let Some(last) = self.last_request {
            let elapsed = last.elapsed();
            if elapsed < MIN_REQUEST_INTERVAL {
                tokio::time::sleep(MIN_REQUEST_INTERVAL - elapsed).await;
            }
        }
        self.last_request = Some(Instant::now());
    }

    async fn single(&self, source: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        let body = format!(
            "sl={}&tl={}&q={}",
            urlencoding::encode(source_lang),
            urlencoding::encode(target_lang),
            urlencoding::encode(source)
        );

        let response = self
            .client
            .post("https://translate.google.com/translate_a/single?client=at&dt=t&dt=rm&dj=1")
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=utf-8",
            )
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Google responded with {status}").into());
        }

        let json: serde_json::Value = serde_json::from_str(&response.text().await?)?;
        let translated: String = json["sentences"]
            .as_array()
            .ok_or("Unexpected response: missing sentences")?
            .iter()
            .filter_map(|sentence| sentence["trans"].as_str())
            .collect();
        Ok(translated)
    }

    async fn batch(&self, source: &str, source_lang: &str, target_lang: &str) -> Result<String> {
        let inner =
            serde_json::json!([[source, source_lang, target_lang, false], [null]]).to_string();
        let freq = serde_json::json!([[[BATCH_RPC_ID, inner, null, "0"]]]).to_string();
        let body = format!("f.req={}&", urlencoding::encode(&freq));

        let req_id = 1000
            + std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.subsec_nanos() % 9000)
                .unwrap_or(0);
        let url = format!(
            "https://translate.google.com/_/TranslateWebserverUi/data/batchexecute?rpcids={BATCH_RPC_ID}\
             &source-path=%2F&f.sid=&bl=&hl=en-US&soc-app=1&soc-platform=1&soc-device=1&_reqid={req_id}&rt=c"
        );

        let response = self
            .client
            .post(url)
            .header(
                "Content-Type",
                "application/x-www-form-urlencoded;charset=UTF-8",
            )
            .body(body)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(format!("Google (batch) responded with {status}").into());
        }

        // body starts with the anti-XSSI prefix `)]}'`, followed by length-prefixed JSON lines
        let text = response.text().await?;
        for line in text.lines().filter(|line| line.starts_with('[')) {
            let Ok(chunk) = serde_json::from_str::<serde_json::Value>(line) else {
                continue;
            };
            let Some(entries) = chunk.as_array() else {
                continue;
            };
            for entry in entries {
                if entry[0].as_str() != Some("wrb.fr") {
                    continue;
                }
                let payload = entry[2]
                    .as_str()
                    .ok_or("Batch translation rejected by the server")?;
                let translation: serde_json::Value = serde_json::from_str(payload)?;
                let node = &translation[1][0][0];
                return match node[5].as_array() {
                    Some(parts) => Ok(parts
                        .iter()
                        .filter_map(|part| part[0].as_str())
                        .filter(|part| !part.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ")),
                    None => node[0]
                        .as_str()
                        .map(str::to_string)
                        .ok_or_else(|| "Unexpected batch response".into()),
                };
            }
        }
        Err("Unexpected batch response: missing translation".into())
    }
}
