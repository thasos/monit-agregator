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

    // TODO use :
    //     Response::builder()
    //   .header("my-custom-header", "some-value")
    //   .body("and a custom body")
    // TODO add "refresh all" button, and print the value of refresh_period
    const CARGO_PKG_VERSION: Option<&str> = option_env!("CARGO_PKG_VERSION");
    let html_head = r#"<!DOCTYPE html>
<html lang="us">
    <head>
        <meta charset="UTF-8" />
        <meta name="viewport" content="width=device-width" />
        <title>Monit Agregator</title>
        <!-- css from w3c -->
        <link rel="stylesheet" href="css/w3.css">
        <link rel="stylesheet" href="css/w3-theme-dark-grey.css">
        <!-- disable cache -->
        <meta http-equiv="Cache-Control" content="no-cache, no-store, must-revalidate">
        <meta http-equiv="Pragma" content="no-cache">
        <meta http-equiv="Expires" content="0">
    </head>
    <body class="w3-theme-dark">
            <div class="w3-panel">
            <h1>Monit-Agregator</h1>
            </div>"#;
    let html_foot = format!(
        r#"<br /><div class="w3-twothird w3-panel">version : {}<br /><a href="https://codeberg.org/thasos/monit-agregator">sources</a></div></body></html>"#,
        CARGO_PKG_VERSION.unwrap_or("version not found")
    );

    // css
    let css_w3 = include_bytes!("css/w3.css");
    let css_w3_theme = include_bytes!("css/w3-theme-dark-grey.css");
    let css = warp::path!("css" / String).map(move |subpath: String| {
        info!("client reqwested css '{}'", subpath);
        match subpath.as_str() {
            "w3.css" => warp::reply::with_header(
                String::from_utf8_lossy(css_w3),
                "content-type",
                "text/css",
            ),
            "w3-theme-dark-grey.css" => warp::reply::with_header(
                String::from_utf8_lossy(css_w3_theme),
                "content-type",
                "text/css",
            ),
            // TODO comment gérer le 404 ici ???
            _ => warp::reply::with_header(
                String::from_utf8_lossy(css_w3),
                "content-type",
                "text/css",
            ),
        }
    });

    // home
    let homepage = warp::path::end().map(move || {
        info!("homepage reqwested");
        let pretty_status = rx.borrow().to_owned();
        let html_body = format!(
            r#"
            "<div class="w3-twothird w3-panel">
                <table class="w3-table w3-centered">
                    <tr>
                        {}
                    </tr>
                </table>
            </div>"#,
            pretty_status
        );
        warp::reply::html(format!("{html_head}\n{html_body}\n{html_foot}"))
    });

    let other_paths = warp::path!(String).map(move |subpath: String| {
        // move is used by hosts_urls
        info!("client reqwested host '{}'", subpath);
        match subpath.as_str() {
            "favicon.ico" => warp::reply::html(String::from("no favicon yet\n")),
            // TODO create help page
            "help" => warp::reply::html("HEEEEEELP\n".to_owned()),
            _ => {
                // must clone hosts_urls in this scope because get_monit take ownership
                warp::reply::html(String::from("not found\n"))
            }
        }
    });

    info!("listening on {}:{}", bind_address.ip(), bind_address.port());
    let routes = monit_proxy.or(homepage).or(other_paths).or(css);
    warp::serve(routes).run(bind_address).await;
}
