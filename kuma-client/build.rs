use serde::{Deserialize, Serialize};
use std::{env, fs::File, io::Write, path::Path};

#[derive(Serialize, Deserialize)]
struct TimeZoneInfo {
    #[serde(rename = "tzCode")]
    pub tz_code: String,
    pub label: String,
    pub name: String,
    pub utc: String,
}

fn generate_timezones() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("timezones.rs");

    let timezones: Vec<TimeZoneInfo> =
        serde_json::from_reader(std::fs::File::open("timezones.json").unwrap()).unwrap();

    let mut file = File::create(&dest_path).unwrap();

    writeln!(file, "#[derive(Clone, Debug, PartialEq, Eq)]").unwrap();
    writeln!(file, "#[allow(non_camel_case_types)]").unwrap();
    writeln!(file, "pub enum TimeZone {{").unwrap();

    for tz in &timezones {
        writeln!(
            file,
            "  {},",
            tz.tz_code.replace("/", "_").replace("-", "_")
        )
        .unwrap();
    }

    writeln!(file, "}}").unwrap();

    writeln!(file, "impl TimeZone {{").unwrap();

    let props: &[(&str, Box<dyn Fn(&TimeZoneInfo) -> String>)] = &[
        (
            "identifier",
            Box::new(|tz: &TimeZoneInfo| tz.tz_code.to_owned()),
        ),
        ("name", Box::new(|tz: &TimeZoneInfo| tz.name.to_owned())),
        ("label", Box::new(|tz: &TimeZoneInfo| tz.label.to_owned())),
        (
            "utc_offset",
            Box::new(|tz: &TimeZoneInfo| tz.utc.to_owned()),
        ),
    ];

    for prop in props {
        writeln!(file, "  pub fn {}(&self) -> &str {{", prop.0).unwrap();
        writeln!(file, "    match self {{").unwrap();

        for tz in &timezones {
            writeln!(
                file,
                "      TimeZone::{} => r#\"{}\"#,",
                tz.tz_code.replace("/", "_").replace("-", "_"),
                prop.1(tz)
            )
            .unwrap();
        }

        writeln!(file, "    }}").unwrap();
        writeln!(file, "  }}").unwrap();
    }

    writeln!(
        file,
        "  pub fn from_str(identifier: impl AsRef<str>) -> Option<Self> {{"
    )
    .unwrap();
    writeln!(file, "    match identifier.as_ref() {{").unwrap();

    for tz in &timezones {
        writeln!(
            file,
            "      r#\"{}\"# => Some(TimeZone::{}),",
            tz.tz_code,
            tz.tz_code.replace("/", "_").replace("-", "_"),
        )
        .unwrap();
    }
    writeln!(file, "      _ => None,",).unwrap();

    writeln!(file, "    }}").unwrap();
    writeln!(file, "  }}").unwrap();

    writeln!(file, "}}").unwrap();
    println!("cargo:rerun-if-changed=timezones.rs");
}

fn main() -> shadow_rs::SdResult<()> {
    println!("cargo:rerun-if-changed=build.rs");

    generate_timezones();

    shadow_rs::new()
}
