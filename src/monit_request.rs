use crate::conf::MonitHosts;

use log::{error, info, warn};
use std::process;
use std::sync::mpsc::channel;
use std::thread;
use std::time::Duration;

/// Spawn a thread with a blocking reqwest to the monit url
/// and return the html text response in a String
pub fn get_monit(hosts_urls: Vec<MonitHosts>, hostname: &str) -> String {
    // simple channel for body monit response
    let (tx, rx) = channel();

    // need ownership... why ?
    let hostname = hostname.to_owned();

    // create thread for new reqwest
    thread::spawn(move || {
        // verify if host reqwested is in config
        debug!("searching '{}' in conf", &hostname);
        if hosts_urls.iter().any(|host| host.name == hostname) {
            debug!("host '{}' found in conf", &hostname);
            // TODO use macro (filter ?)
            for host in hosts_urls {
                if host.name == hostname {
                    let monit_url = host.url;
                    info!("reqwesting '{}' ({})", host.name, monit_url);

                    // let's reqwest the host !
                    let client = match reqwest::blocking::Client::builder()
                        // TODO set timeout from conf
                        .timeout(Duration::from_secs(5))
                        .build()
                    {
                        Ok(client) => client,
                        Err(_) => {
                            error!("unable to create reqwest client, this should not happen");
                            process::exit(1);
                        }
                    };
                    match client.get(&monit_url).send() {
                        Ok(response) => {
                            let response = response.text();
                            let val = match response {
                                Ok(response) => {
                                    debug!("response from monit ok, forward response body");
                                    format!("{}\n", response)
                                }
                                // if monit does not return a 200 HTTP, proxy respond 500
                                Err(_) => {
                                    warn!("response from monit KO, response generic 500");
                                    format!("HTTP 500 : error from monit on host {}\n", &hostname)
                                }
                            };
                            match tx.send(val) {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("thread channel error : {}", e);
                                    process::exit(1);
                                }
                            };
                        }
                        // if we cannot send http reqwest for some reason
                        Err(_) => {
                            let msg = format!("error reqwesting {}", monit_url);
                            warn!("{}", msg);
                            match tx.send(msg) {
                                Ok(_) => (),
                                Err(e) => {
                                    error!("thread channel error : {}", e);
                                    process::exit(1);
                                }
                            };
                        }
                    };
                }
            }
        // host not found in config
        } else {
            tx.send(format!("no host \"{}\" found in conf", &hostname))
                .unwrap();
        }
    });

    match rx.recv() {
        Ok(body) => body,
        Err(_) => String::from("Error reqwesting\n"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    //use mockito;
    use mockito::mock;

    #[test]
    fn test_get_monit() {
        // create mock content, path, etc...
        let content_tested = "blablabla";
        let _mt = mock("GET", "/")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(content_tested)
            .create();

        // call get_monit
        let hostname = "tarsis";
        let url = &mockito::server_url();
        let hosts_urls = vec![MonitHosts {
            name: hostname.to_owned(),
            url: url.to_owned(),
            public_url: None,
        }];
        assert_eq!(
            get_monit(hosts_urls, hostname),
            format!("{}\n", content_tested)
        );
    }
}
