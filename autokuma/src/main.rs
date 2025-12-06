use crate::util::ResultOrDie;
use ::config::{Config, Environment, File, FileFormat};
use flexi_logger::{
    AdaptiveFormat, Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming,
};
use kuma_client::build::SHORT_VERSION;
use kuma_client::util::ResultLogger;
use owo_colors::{
    colors::{self, xterm},
    Style,
};
use serde_json::json;
use std::{
    hash::{DefaultHasher, Hash as _, Hasher as _},
    sync::Arc,
};

include!("mod.rs");

const BANNER: &str = r"                                                        
                .:::.                                      .:::.                
              .===-====:                                :-===--==:              
             .==.    .:==-.        ..........         :==-.    .==:             
             -=-        :===--====================---==:        -==             
             -=-          :===-..              ..:===-          :==             
             -=-            ::                    .-.           -==             
             :==                                                ==-             
              ==.                                              .==.             
             :==-                                              -==-             
            .====.                                             ====-            
            ==-                                                  .==:           
           :==                                                    ===           
           -==                                                    -==           
           -==                                                    :==           
           -==               ..        ...       ..               -==           
           .==.             :===     -=====.    ====              ==-           
            ===              .:.   :==-  :==-    ::              :==.           
            .==:                  :==:    .==-                  .==:            
             .==.                :==:      .==-                .==-             
              .==:              .==:        .==:              .==:              
               .==-             ==-          :==             :==:               
                .-==:          :==            -=-          .-==.                
                  .===.        ==.   .::::..   ==.       .-==:                  
                    :===.     :=-  ==========. :==     .-==:                    
                      .===:   ==.  -=========  .==.  .-==:                      
                        .-==-:==    .======:    ==-:===:                        
                           :-===:      ...     .====:.                          
                              :==-.          .-==:                              
                                :====---:--====:                                
                                   .::----::.                                   
                            _           _  __                         
              /\           | |         | |/ /                         
             /  \    _   _ | |_   ___  | ' /  _   _  _ __ ___    __ _ 
            / /\ \  | | | || __| / _ \ |  <  | | | || '_ ` _ \  / _` |
           / ____ \ | |_| || |_ | (_) || . \ | |_| || | | | | || (_| |
          /_/    \_\ \__,_| \__| \___/ |_|\_\ \__,_||_| |_| |_| \__,_|  
";

fn level_style(level: log::Level) -> Style {
    match level {
        log::Level::Error => Style::new().fg::<colors::Red>(),
        log::Level::Warn => Style::new().fg::<colors::Yellow>(),
        log::Level::Info => Style::new().fg::<colors::Cyan>(),
        log::Level::Debug => Style::new().fg::<colors::White>(),
        log::Level::Trace => Style::new().fg::<colors::BrightBlack>(),
    }
}

#[test]
fn test_level_style() {
    println!(
        "{} {} {} {} {}",
        level_style(log::Level::Error).style("Error"),
        level_style(log::Level::Warn).style("Warn"),
        level_style(log::Level::Info).style("Info"),
        level_style(log::Level::Debug).style("Debug"),
        level_style(log::Level::Trace).style("Trace"),
    );
}

fn module_style(module: &str) -> Style {
    let mut hash = DefaultHasher::default();
    module.hash(&mut hash);
    let index = (hash.finish() as usize) % 10;

    match index {
        0 => Style::new().fg::<colors::Cyan>(),
        1 => Style::new().fg::<colors::Green>(),
        2 => Style::new().fg::<xterm::LightScreaminGreen>(),
        3 => Style::new().fg::<colors::Blue>(),
        4 => Style::new().fg::<xterm::DarkAnakiwaBlue>(),
        5 => Style::new().fg::<colors::Magenta>(),
        6 => Style::new().fg::<xterm::FlushOrange>(),
        7 => Style::new().fg::<xterm::LightHeliotrope>(),
        8 => Style::new().fg::<xterm::RoseofSharonOrange>(),
        _ => Style::new().fg::<xterm::LavenderRose>(),
    }
}

#[test]
fn test_module_style() {
    for i in 0..10 {
        let mut rng = rand::rng();
        let msg = loop {
            let mut msg = String::new();
            for _ in 0..32 {
                msg.push(rand::Rng::sample(&mut rng, rand::distr::Alphanumeric) as char);
            }

            let mut hash = DefaultHasher::default();
            msg.hash(&mut hash);
            let index = hash.finish() as usize % 10;

            if index == i {
                break msg;
            }
        };

        println!(
            "{}",
            module_style(&msg).style(format!("Module style {}", i + 1))
        );
    }
}

fn create_logger(config: &Arc<crate::config::Config>) -> LoggerHandle {
    let format = AdaptiveFormat::Custom(
        |write, now, record| {
            write!(
                write,
                "{} [{}] {}: {}",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                record.target(),
                record.level().to_string(),
                record.args().to_string()
            )
        },
        |write, now, record| {
            write!(
                write,
                "{} [{}] {}: {}",
                now.format("%Y-%m-%d %H:%M:%S%.3f"),
                module_style(record.target()).style(record.target()),
                level_style(record.level()).style(record.level().to_string()),
                record.args().to_string()
            )
        },
    );

    let mut builder = Logger::try_with_env_or_str("info, kube_runtime=error")
        .unwrap()
        .set_palette("196;208;14;7;8".to_owned())
        .adaptive_format_for_stderr(format)
        .adaptive_format_for_stdout(format);

    if let Some(log_dir) = config.log_dir.as_ref() {
        builder = builder
            .log_to_file(FileSpec::default().directory(log_dir))
            .append()
            .rotate(
                Criterion::Size(1_000_000),
                Naming::NumbersDirect,
                Cleanup::KeepLogAndCompressedFiles(1, 5),
            )
            .duplicate_to_stderr(Duplicate::All);
    }

    return builder.start().unwrap();
}

#[cfg(feature = "tokio-console")]
fn init_console_subscriber() {
    console_subscriber::init();
}

#[cfg(not(feature = "tokio-console"))]
fn init_console_subscriber() {}

#[tokio::main()]
async fn main() {
    init_console_subscriber();

    let config: Arc<crate::config::Config> = Arc::new(
        Config::builder()
            .add_source(File::from_str(
                &serde_json::to_string(
                    &json!({"kuma": {"tls": {}}, "docker": {}, "files": {}, "kubernetes": {}}),
                )
                .unwrap(),
                FileFormat::Json,
            ))
            .add_source(
                File::with_name(
                    &dirs::config_local_dir()
                        .map(|dir| {
                            dir.join("autokuma")
                                .join("config")
                                .to_string_lossy()
                                .to_string()
                        })
                        .unwrap_or_default(),
                )
                .required(false),
            )
            .add_source(File::new("autokuma.toml", FileFormat::Toml).required(false))
            .add_source(File::new("autokuma.yaml", FileFormat::Yaml).required(false))
            .add_source(File::new("autokuma.json", FileFormat::Json).required(false))
            .add_source(
                Environment::with_prefix("AUTOKUMA")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()
            .print_error(|e| format!("Unable to load config: {}", e))
            .and_then(|config| config.try_deserialize())
            .print_error(|e| format!("Invalid config: {}", e))
            .unwrap_or_die(1),
    );

    let logger = create_logger(&config);

    println!("{}{:>70}", BANNER, SHORT_VERSION);

    let mut sync = sync::Sync::new(config)
        .log_error(std::module_path!(), |e| format!("Invalid config: {}", e))
        .unwrap_or_die(1);

    sync.run().await;

    logger.shutdown();
}
