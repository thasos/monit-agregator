mod conf;
mod http_server;
mod monit_request;
mod status_generation;

#[macro_use]
extern crate log; // env_logger
use std::thread;
use tokio::sync::watch;

#[tokio::main]
async fn main() {
    // build conf
    //conf::set_loglevel();
    let conf = conf::get_conf();
    const CARGO_PKG_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    info!(
        "starting up... version={}",
        CARGO_PKG_VERSION.unwrap_or("version not found")
    );

    let bind_address = conf.bind_address;
    let hosts_urls = conf.hosts_urls;
    let wait_period = conf.wait_period;

    // channel for status homepage creation
    // TODO homepage vide / build skeleton here
    let (tx, rx) = watch::channel(String::from("homepage build in progress"));

    let server_hosts_urls = hosts_urls.clone();
    thread::spawn(move || {
        http_server::serve_http(bind_address, server_hosts_urls, rx);
    });

    // https://stackoverflow.com/questions/62536566/how-can-i-create-a-tokio-runtime-inside-another-tokio-runtime-without-getting-th
    let generation_loop_status_thread = thread::spawn(move || {
        status_generation::generate_home(hosts_urls, tx, wait_period);
    })
    .join();
    if generation_loop_status_thread.is_err() {
        error!("unable to start loop generation status")
    };
}
