use config::Config;
use log::LevelFilter;
use serde::{Deserialize, Serialize};
use std::env;
use std::net::{Ipv4Addr, SocketAddrV4};
use std::time::Duration;

pub struct MonitAgregatorConfig {
    pub bind_address: SocketAddrV4,
    pub hosts_urls: Vec<MonitHosts>,
    pub wait_period: Duration,
}

use clap::Parser;
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(
        short,
        long,
        default_value = "Settings.yaml",
        help = "config file path"
    )]
    config: String,

    #[clap(
        short,
        long,
        default_value = "info",
        help = "possible values : error warn info debug trace [default: info]"
    )]
    loglevel: LevelFilter,

    #[clap(short, long, default_value = "127.0.0.1", help = "bind ip to listen")]
    bind_ip: String,

    #[clap(short, long, default_value = "3030", help = "port to listen")]
    port: u16,

    #[clap(
        short,
        long,
        default_value = "60",
        help = "sleep duration between refresh of monit status for each instance"
    )]
    refresh_period: u64,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct MonitHosts {
    pub name: String,
    pub url: String,
    pub public_url: Option<String>,
}

pub fn get_conf() -> MonitAgregatorConfig {
    // priority is clap, then config file, then default
    let args = Args::parse();
    init_logger(args.loglevel);

    let default_ip = Ipv4Addr::new(127, 0, 0, 1);
    let default_port = 3030;
    let bind_address: SocketAddrV4;

    let default_wait_period = 60;
    let wait_period: Duration = Duration::from_secs(default_wait_period);

    let mut hosts_urls: Vec<MonitHosts> = Vec::new();

    // default config_file_path
    let config_file_path = args.config;

    // get conf from file and env
    match Config::builder()
        .add_source(config::File::with_name(&config_file_path))
        .add_source(config::Environment::with_prefix("MONAGR"))
        .build()
    {
        Ok(settings) => {
            info!("config file found : {}", config_file_path);

            // socket to liston
            let parsed_clap_ip = match args.bind_ip.parse::<Ipv4Addr>() {
                Ok(parsed_clap_ip) => parsed_clap_ip,
                Err(_) => {
                    error!("error parsing ip, set to default {:?}", default_ip);
                    default_ip
                }
            };
            let ip = if parsed_clap_ip == default_ip {
                match settings.get_string("ip") {
                    Ok(ip) => match ip.parse() {
                        Ok(parsed_ip) => parsed_ip,
                        Err(_) => {
                            error!("error parsing ip, set to default {:?}", default_ip);
                            default_ip
                        }
                    },
                    Err(_) => default_ip,
                }
            } else {
                parsed_clap_ip
            };

            let port: u16 = if args.port == default_port {
                match settings.get_int("port") {
                    Ok(port) => port as u16,
                    Err(_) => default_port,
                }
            } else {
                args.port
            };
            bind_address = SocketAddrV4::new(ip, port);

            // loop stats generation period
            let wait_period = if args.refresh_period == default_wait_period {
                match settings.get_int("wait_period") {
                    Ok(wait_period) => Duration::from_secs(wait_period as u64),
                    Err(_) => Duration::from_secs(default_wait_period),
                }
            } else {
                Duration::from_secs(args.refresh_period as u64)
            };

            debug!("refresh monit status period : {:?}", wait_period);

            // build hosts and urls
            if let Ok(hosts_list) = settings.get_array("hosts") {
                for host in hosts_list {
                    if let Ok(host_and_urls) = host.clone().try_deserialize::<MonitHosts>() {
                        hosts_urls.push(host_and_urls);
                    };
                }
            };
        }
        Err(e) => {
            warn!("no config file found ({}), starting but do nothing", e);
            bind_address = SocketAddrV4::new(default_ip, default_port);
        }
    };
    MonitAgregatorConfig {
        bind_address,
        hosts_urls,
        wait_period,
    }
}

pub fn init_logger(loglevel: LevelFilter) {
    let mut logbuilder = env_logger::Builder::from_default_env();
    logbuilder.target(env_logger::Target::Stdout);

    // if RUST_LOG is setted, and `-l` arg is not provided or default, use RUST_LOG
    if loglevel == LevelFilter::Info {
        if env::var("RUST_LOG").is_err() {
            logbuilder.filter_level(loglevel);
        }
    } else {
        logbuilder.filter_level(loglevel);
    }

    // TODO afficher les logs des dependances ou pas ?
    logbuilder
        .filter(Some("reqwest::"), log::LevelFilter::Off)
        .filter(Some("hyper::"), log::LevelFilter::Off)
        .filter(Some("warp::"), log::LevelFilter::Off);

    match logbuilder.try_init() {
        Ok(_) => (),
        Err(_) => eprintln!("unable to initiate pretty logging"),
    };
    debug!("loglevel : {:?}", loglevel);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_conf() {
        let conf = get_conf();
        assert_eq!(conf.bind_address.ip(), &Ipv4Addr::new(0, 0, 0, 0));
        assert_eq!(conf.bind_address.port(), 3030 as u16);
    }
}
