use std::sync::Arc;

use util::ResultLogger;

mod config;
mod error;
mod kuma;
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

    let config = Arc::new(
        confique::Config::builder()
            .env()
            .file("config.toml")
            .load()
            .log_error(|e| format!("Invalid config: {}", e))
            .unwrap(),
    );

    let sync = sync::Sync::new(config);
    sync.run().await;
}
