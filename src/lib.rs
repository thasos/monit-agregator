use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::process;
use std::sync::mpsc;
use std::thread;
use warp::Filter;

pub struct BindAddress {
    pub port: u16,
    pub ip: Ipv4Addr,
}
impl BindAddress {
    pub fn default() -> BindAddress {
        BindAddress {
            port: 3030,
            ip: Ipv4Addr::new(127, 0, 0, 1),
        }
    }
    pub fn new(ip: &str, port: u16) -> BindAddress {
        let ip: Ipv4Addr = match ip.parse() {
            Ok(ip) => ip,
            Err(e) => {
                eprintln!("Error parsing IP '{}' : {}", ip, e);
                process::exit(1);
            }
        };
        BindAddress { port, ip }
    }
}

/// http server, need bind addr in conf, and hosts_urls for mapping
#[tokio::main]
pub async fn serve_http(bind_ip: Ipv4Addr, bind_port: u16, hosts_urls: Vec<(String, String)>) {
    // on vous sert quoi aujourd'hui ?
    let monit_srv = warp::path!("monit" / String).map(move |hostname: String| {
        // le move sert à hosts_urls
        println!("log: req {}", hostname);
        // return monit html in plain text
        if hostname == "favicon.ico" {
            println!("log: req favicont");
            warp::reply::html("no favico".to_string())
        } else {
            println!("log: req {}", hostname);
            let hosts_urls = hosts_urls.clone(); // faut cloner hosts_urls dans ce scope car get_monit prend l'ownership
            warp::reply::html(get_monit(hosts_urls, hostname.as_str()))
        }
    });
    // bind, serve and protect
    let bind_socket = SocketAddr::new(IpAddr::V4(bind_ip), bind_port);
    println!("log: listening on {}:{}", bind_ip, bind_port);
    warp::serve(monit_srv).run(bind_socket).await;
}

/// Spawn a thread with a blocking reqwest to the monit url
/// and return the html text response in a String
pub fn get_monit(hosts_urls: Vec<(String, String)>, hostname: &str) -> String {
    let (tx, rx) = mpsc::channel();
    let hostname = hostname.to_string();
    thread::spawn(move || {
        println!("log: searching {} in list", hostname);
        for host in hosts_urls {
            if host.0 == hostname {
                println!("log: host {} found", host.0);
                let monit_url = host.1;
                println!("log: reqwesting {}", monit_url);
                match reqwest::blocking::get(&monit_url) {
                    Ok(response) => {
                        let response = response.text();
                        //let response = reqwest::blocking::get(monit_url).unwrap().text();
                        let val = match response {
                            Ok(response) => {
                                println!("log: get monit status ok");
                                response
                            }
                            Err(_) => {
                                println!("log: get monit status ERR");
                                String::from("HTTP 500 : error from monit")
                            }
                        };
                        tx.send(val).unwrap();
                    }
                    Err(_) => {
                        let msg = format!("log: error reqwesting {}", monit_url);
                        println!("{}", msg);
                        tx.send(msg).unwrap();
                    }
                };
                // TODO si hostame pas trouvé, faut renvoyer un truc comme ça...
                //} else {
                //    println!("log: host {} not found in list", hostname);
                //    tx.send("I'm not an open proxy...".to_string()).unwrap();
            }
        }
    });
    match rx.recv() {
        Ok(body) => body,
        Err(_) => "Error reqwesting".to_string(),
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}
