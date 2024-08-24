use crate::cli::Cli;
use clap::ValueEnum;
use futures_util::future::join_all;
use inkjet::{
    constants::HIGHLIGHT_NAMES, formatter::Formatter, tree_sitter_highlight::HighlightEvent,
    Highlighter, InkjetError,
};
use kuma_client::Config;
use owo_colors::Style;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{collections::HashMap, path::PathBuf};
use tap::Pipe;
use tokio::task;

pub(crate) type Result<T> = kuma_client::error::Result<T>;

pub(crate) async fn connect(config: &Config, cli: &Cli) -> kuma_client::Client {
    kuma_client::Client::connect(config.clone())
        .await
        .unwrap_or_die(cli)
}

#[derive(ValueEnum, Clone, Debug)]
pub(crate) enum OutputFormat {
    Json,
    Yaml,
}

pub(crate) trait PrintResult {
    fn print_result(self, cli: &Cli);
}

impl<T> PrintResult for Result<T>
where
    T: Sized + Serialize,
{
    fn print_result(self, cli: &Cli) {
        let value = self.unwrap_or_die(cli);
        print_value(&value, cli);
    }
}

pub(crate) trait ResultOrDie<T> {
    fn unwrap_or_die(self, cli: &Cli) -> T;
}

impl<T, E> ResultOrDie<T> for std::result::Result<T, E>
where
    E: ToString,
{
    fn unwrap_or_die(self, cli: &Cli) -> T {
        match self {
            Ok(t) => t,
            Err(error) => {
                print_value(&json!({"error": error.to_string()}), cli);
                std::process::exit(1)
            }
        }
    }
}

pub(crate) fn print_value<T>(value: &T, cli: &Cli)
where
    T: Serialize,
{
    let str = match (&cli.output_format, &cli.output_pretty) {
        (OutputFormat::Json, true) => Highlighter::new()
            .highlight_to_string(
                inkjet::Language::Json,
                &ColorPrinter::new(),
                serde_json::to_string_pretty(value).unwrap(),
            )
            .unwrap(),
        (OutputFormat::Json, false) => serde_json::to_string(value).unwrap(),
        (OutputFormat::Yaml, true) => Highlighter::new()
            .highlight_to_string(
                inkjet::Language::Yaml,
                &ColorPrinter::new(),
                serde_yaml::to_string(value).unwrap(),
            )
            .unwrap(),
        (OutputFormat::Yaml, false) => serde_yaml::to_string(value).unwrap(),
    };

    print!("{}", str);
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
pub(crate) enum OneOrMany<T> {
    One(T),
    Many(Vec<T>),
}
pub(crate) async fn load_files<T>(file: &Vec<PathBuf>, cli: &Cli) -> Vec<T>
where
    T: Send + for<'de> serde::Deserialize<'de> + 'static,
{
    file.into_iter()
        .map(|file| load_file(file, cli))
        .collect::<Vec<_>>()
        .pipe(|futures| join_all(futures))
        .await
        .into_iter()
        .flatten()
        .collect()
}

pub(crate) async fn load_file<T>(file: &PathBuf, cli: &Cli) -> Vec<T>
where
    T: Send + for<'de> serde::Deserialize<'de> + 'static,
{
    let file_clone = file.clone();
    let cli_clone = cli.clone();

    let result = task::spawn_blocking(move || {
        if file_clone.to_string_lossy() == "-" {
            serde_json::from_reader(std::io::stdin()).unwrap_or_die(&cli_clone)
        } else {
            serde_json::from_reader(std::fs::File::open(&file_clone).unwrap_or_die(&cli_clone))
                .unwrap_or_die(&cli_clone)
        }
    })
    .await
    .unwrap_or_die(cli);

    match result {
        OneOrMany::One(x) => vec![x],
        OneOrMany::Many(x) => x,
    }
}

pub(crate) trait CollectOrUnwrap: Iterator {
    fn collect_or_unwrap(self) -> OneOrMany<Self::Item>
    where
        Self: Sized,
    {
        let vec = self.collect::<Vec<_>>();
        match vec.len() {
            1 => OneOrMany::One(vec.into_iter().next().unwrap()),
            _ => OneOrMany::Many(vec),
        }
    }
}

impl<T: Iterator> CollectOrUnwrap for T {}

#[derive(Clone)]
struct Theme {
    pub default_style: Style,
    pub styles: HashMap<String, Style>,
}

impl Theme {
    pub fn new(default_style: Style) -> Self {
        Self {
            default_style,
            styles: HashMap::new(),
        }
    }

    pub fn resolve_style(&self, name: impl AsRef<str>) -> &Style {
        let name = name.as_ref();

        if let Some(style) = self.styles.get(name) {
            return style;
        }

        if let Some(pos) = name.rfind('.') {
            return self.resolve_style(&name[0..pos]);
        }

        &self.default_style
    }

    pub fn add_style(&mut self, name: impl Into<String>, style: impl Into<Style>) -> &mut Self {
        self.styles.insert(name.into(), style.into());

        self
    }
}

pub struct ColorPrinter {
    theme: Theme,
    supports_color: bool,
}

impl ColorPrinter {
    pub fn new() -> Self {
        Self {
            theme: Theme::new(Style::new())
                .add_style("string", Style::new().green())
                .add_style("variable", Style::new().blue().bold())
                .add_style("keyword", Style::new().blue().bold())
                .add_style("constant.numeric", Style::new().blue())
                .add_style("constant.builtin", Style::new().purple())
                .clone(),

            supports_color: std::env::var("FORCE_COLOR")
                .map(|force_color| force_color != "0")
                .ok()
                .unwrap_or_else(|| {
                    supports_color::on_cached(supports_color::Stream::Stdout)
                        .map(|level| level.has_basic)
                        .unwrap_or(false)
                }),
        }
    }
}

struct Prefix<'a>(&'a Style);
struct Suffix<'a>(&'a Style);

impl std::fmt::Display for Prefix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt_prefix(f)
    }
}

impl std::fmt::Display for Suffix<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.0.fmt_suffix(f)
    }
}

impl Formatter for ColorPrinter {
    fn write<W>(
        &self,
        source: &str,
        writer: &mut W,
        event: HighlightEvent,
    ) -> std::result::Result<(), InkjetError>
    where
        W: std::fmt::Write,
    {
        match event {
            HighlightEvent::Source { start, end } => {
                let span = source
                    .get(start..end)
                    .expect("Source bounds should be in bounds!");
                writer.write_str(span)?;
            }
            HighlightEvent::HighlightStart(idx) => {
                if self.supports_color {
                    let name = HIGHLIGHT_NAMES[idx.0];
                    let style = self.theme.resolve_style(name);
                    write!(writer, "{}", Prefix(&style))?;
                }
            }
            HighlightEvent::HighlightEnd => {
                if self.supports_color {
                    write!(writer, "{}", Suffix(&Style::new().white()))?;
                }
            }
        }

        Ok(())
    }
}
