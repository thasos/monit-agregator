use config::Config;
use monit_agregator::BindAddress;
// TODO clap pour options

/// get conf from file for binding ip and port
/// and host list with url of monit for each
fn get_conf() -> (BindAddress, Vec<(String, String)>) {
    // TODO faire une struct Config...
    let bind_address: BindAddress;
    let mut hosts_urls: Vec<(String, String)> = Vec::new();

    // default config_file_path
    let config_file_path = "Settings.yaml";

    // get conf from file and env
    match Config::builder()
        .add_source(config::File::with_name(config_file_path))
        .add_source(config::Environment::with_prefix("MONAGR"))
        .build()
    {
        Ok(settings) => {
            // 🤮🤮🤮🤮🤮🤮🤮🤮🤮🤮
            let debug_mode = settings.get_string("debug").unwrap();
            if !debug_mode.is_empty() {
                println!("env: debug = {}", debug_mode);
            } else {
                println!("env: debug = false");
            }

            // récup ip et port pour en faire un bind
            let ip = settings.get_string("ip").unwrap();
            let port: u16 = settings.get_int("port").unwrap() as u16;
            bind_address = BindAddress::new(ip.as_str(), port);

            // hosts and urls
            // TODO can refactor chaining `get_array` and `into_table` ?
            if let Ok(hosts_list) = settings.get_array("hosts") {
                for host in hosts_list {
                    if let Ok(host_and_url) = host.into_table() {
                        for (hostname, url) in &host_and_url {
                            //println!("{}: {}", hostname, url);
                            hosts_urls.push((hostname.to_string(), url.to_string()));
                        }
                    };
                }
            };
        }
        Err(e) => {
            println!("warn: could not open {} : {}", config_file_path, e);
            // need default address to liston on
            bind_address = BindAddress::default();
            // if no host / urls found, server start but no with proxy...
        }
    };
    (bind_address, hosts_urls)
}

fn main() {
    let conf = get_conf();
    let bind_address = conf.0;
    let hosts_urls = conf.1;

    // let's start server
    monit_agregator::serve_http(bind_address.ip, bind_address.port, hosts_urls);
}
