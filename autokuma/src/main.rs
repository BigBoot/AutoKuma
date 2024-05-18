use ::config::{Config, Environment, File, FileFormat};
use flexi_logger::{Cleanup, Criterion, Duplicate, FileSpec, Logger, LoggerHandle, Naming};
use kuma_client::build::SHORT_VERSION;
use serde_json::json;
use std::sync::Arc;
use util::{ResultLogger, ResultOrDie};

mod config;
mod error;
mod sync;
mod util;

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

fn create_logger(config: &Arc<crate::config::Config>) -> LoggerHandle {
    let mut builder = Logger::try_with_env_or_str("info").unwrap();

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
                &serde_json::to_string(&json!({"kuma": {}, "docker": {}})).unwrap(),
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
            .add_source(File::new("autokuma", FileFormat::Toml).required(false))
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

    let sync = sync::Sync::new(config)
        .log_error(std::module_path!(), |e| format!("Invalid config: {}", e))
        .unwrap_or_die(1);

    sync.run().await;

    logger.shutdown();
}
