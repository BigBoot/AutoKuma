use ::config::{Config, Environment, File, FileFormat};
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

#[tokio::main()]
async fn main() {
    pretty_env_logger::formatted_timed_builder()
        .filter(None, log::LevelFilter::Info)
        .parse_default_env()
        .init();

    println!("{}", BANNER);

    let config: Arc<crate::config::Config> = Arc::new(
        Config::builder()
            .add_source(File::new("config", FileFormat::Toml).required(false))
            .add_source(
                Environment::with_prefix("AUTOKUMA")
                    .separator("__")
                    .prefix_separator("__"),
            )
            .build()
            .log_error(|e| format!("Unable to load config: {}", e))
            .and_then(|config| config.try_deserialize())
            .log_error(|e| format!("Invalid config: {}", e))
            .unwrap_or_die(1),
    );

    let sync = sync::Sync::new(config)
        .log_error(|e| format!("Invalid config: {}", e))
        .unwrap_or_die(1);

    sync.run().await;
}
