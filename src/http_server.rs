use crate::conf::MonitHosts;
use crate::monit_request;

use std::net::SocketAddrV4;
use tokio::sync::watch;
use warp::Filter;

#[tokio::main]
pub async fn serve_http(
    bind_address: SocketAddrV4,
    hosts_urls: Vec<MonitHosts>,
    rx: watch::Receiver<String>,
) {
    // proxy to monit instances
    let monit_proxy = warp::path!("monit" / String).map(move |subpath: String| {
        info!("client reqwested host '{}'", subpath);
        // faut cloner hosts_urls dans ce scope car get_monit prend l'ownership
        let hosts_urls = hosts_urls.clone();
        warp::reply::html(monit_request::get_monit(hosts_urls, subpath.as_str()))
    });

    let html_head = "<!DOCTYPE html>
<html lang=\"us\">
    <head>
        <meta charset=\"UTF-8\" />
        <meta name=\"viewport\" content=\"width=device-width\" />
        <title>Monit Agregator</title>
        <!-- css from w3c -->
        <link rel=\"stylesheet\" href=\"css/w3.css\">
        <link rel=\"stylesheet\" href=\"css/w3-theme-dark-grey.css\">
    </head>
    <body class=\"w3-theme-dark\">";
    let html_foot = "</body></html>";

    let homepage = warp::path::end().map(move || {
        info!("homepage reqwested");
        let message = rx.borrow().to_owned();
        warp::reply::html(format!("{}\n{}\n{}", html_head, message, html_foot))
    });

    let other_paths = warp::path!(String).map(move |subpath: String| {
        // le move sert à hosts_urls
        info!("client reqwested host '{}'", subpath);
        match subpath.as_str() {
            "favicon.ico" => warp::reply::html(String::from("no favicon\n")),
            // TODO créer la page help
            "help" => warp::reply::html("HEEEEEELP\n".to_owned()),
            _ => {
                // faut cloner hosts_urls dans ce scope car get_monit prend l'ownership
                warp::reply::html(String::from("not found\n"))
            }
        }
    });

    info!("listening on {}:{}", bind_address.ip(), bind_address.port());
    let routes = monit_proxy.or(homepage).or(other_paths);
    warp::serve(routes).run(bind_address).await;
}
