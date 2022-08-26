use crate::conf::MonitHosts;
use crate::monit_request;

use std::thread;
use std::time::Duration;
use tokio::sync::watch;

#[derive(Debug, PartialEq)]
struct MonitReport {
    nb_up: Option<i32>,
    nb_down: Option<i32>,
    nb_initialising: Option<i32>,
    nb_unmonitored: Option<i32>,
    total: Option<i32>,
    percent_up: Option<f32>,
}
impl MonitReport {
    fn get_nb_up(&self) -> String {
        match self.nb_up {
            Some(nb) => nb.to_string(),
            None => String::from("unknow"),
        }
    }
    fn get_nb_down(&self) -> String {
        match self.nb_down {
            Some(nb) => nb.to_string(),
            None => String::from("unknow"),
        }
    }
    fn get_nb_initialising(&self) -> String {
        match self.nb_initialising {
            Some(nb) => nb.to_string(),
            None => String::from("unknow"),
        }
    }
    fn get_nb_unmonitored(&self) -> String {
        match self.nb_unmonitored {
            Some(nb) => nb.to_string(),
            None => String::from("unknow"),
        }
    }
    fn get_total(&self) -> String {
        match self.total {
            Some(nb) => nb.to_string(),
            None => String::from("unknow"),
        }
    }
}

fn parse_monit_report(monit_report: &str) -> MonitReport {
    // init report
    let mut report = MonitReport {
        nb_up: None,
        nb_down: None,
        nb_initialising: None,
        nb_unmonitored: None,
        total: None,
        percent_up: None,
    };
    // feed the report
    for line in monit_report.split('\n') {
        // check if line is parsable
        if line.contains(": ") {
            let splitted_line: Vec<&str> = line.split(':').collect(); // ["up", "            85 (100.0%)"]
            let field = splitted_line[0];
            let parsed_value = splitted_line[1].trim_start(); // "85 (100.0%)"
            let splitted_value: Vec<&str> = parsed_value.split(' ').collect(); // ["85", "(100.0%)"]
            let value = splitted_value[0];
            match field {
                "up" => {
                    report.nb_up = match value.parse() {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    };
                    report.percent_up = match splitted_value[1]
                        .replace('(', "")
                        .replace(')', "")
                        .replace('%', "")
                        .parse()
                    {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    };
                }
                "down" => {
                    report.nb_down = match value.parse() {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    }
                }
                "initialising" => {
                    report.nb_initialising = match value.parse() {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    }
                }
                "unmonitored" => {
                    report.nb_unmonitored = match value.parse() {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    }
                }
                "total" => {
                    report.total = match value.parse() {
                        Ok(val) => Some(val),
                        Err(_) => None,
                    }
                }
                &_ => (),
            }
        }
    }
    report
}

fn display_monit_report(report: MonitReport) -> String {
    // percent_up color
    let (color, pretty_percent_up) = match report.percent_up {
        Some(percent) => {
            let color = if percent == 100.0 { "green" } else { "red" };
            (color, percent.to_string())
        }
        None => ("orange", String::from("⚠️ unknow ⚠️")),
    };
    let highlighted_percent_up = format!("<font color={}>{}</font></h2>", color, pretty_percent_up);

    // pretty print in html
    let parsed_report = format!(
        "{}
        up: <b>{}</b><br />
        down: <b>{}</b><br />
        initialising: <b>{}</b><br />
        unmonitored: <b>{}</b><br />
        total: <b>{}</b>",
        highlighted_percent_up,
        report.get_nb_up(),
        report.get_nb_down(),
        report.get_nb_initialising(),
        report.get_nb_unmonitored(),
        report.get_total()
    );
    parsed_report
}

#[tokio::main]
pub async fn generate_home(
    monit_hosts_list: Vec<MonitHosts>,
    tx: watch::Sender<String>,
    wait_period: Duration,
) {
    // build status from hosts urls
    let mut monit_hosts_status: Vec<MonitHosts> = Vec::new();
    for monit_host in monit_hosts_list.iter() {
        monit_hosts_status.push(MonitHosts {
            name: monit_host.name.clone(),
            url: format!("{}/_report", monit_host.url.clone()),
            public_url: monit_host.public_url.clone(),
        });
    }

    // main routine loop
    loop {
        // TODO multiple reqwest in // ?
        // generate status of monit hosts
        debug!("in loop generate_home");
        let mut pretty_status = String::new();
        for (position, host) in monit_hosts_status.clone().iter().enumerate() {
            // retrieve monit status body
            let monit_report = monit_request::get_monit(monit_hosts_status.clone(), &host.name);

            // build pretty status from all hosts
            let parsed_report = parse_monit_report(&monit_report);
            let pretty_report = display_monit_report(parsed_report);

            // replace actual status
            let _old = std::mem::replace(
                &mut monit_hosts_status[position],
                MonitHosts {
                    name: host.name.clone(),
                    url: host.url.clone(),
                    public_url: host.public_url.clone(),
                },
            );
            let url_link = match &host.public_url {
                Some(url) => url.clone(),
                None => monit_hosts_list[position].url.clone(),
            };

            // TODO table display ? (check mobile)
            pretty_status = format!(
                "{}<h2><a href=\"{}\">{}</a> : {}<br />\n",
                &pretty_status,
                url_link,
                host.name.clone(),
                pretty_report,
            );

            // incremetale send
            // TODO display a message "refresh in progress" ?
            match tx.send(pretty_status.clone()) {
                Ok(_) => debug!("homepage status sended to channel"),
                Err(_) => error!("unable to send status on channel"),
            };
        }

        thread::sleep(wait_period);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_monit_report() {
        let report = "up:            85 (100.0%)
down:           2 (0.0%)
initialising:   3 (0.0%)
unmonitored:    4 (0.0%)
total:         94 services";
        let parsed_report = MonitReport {
            nb_up: Some(85),
            nb_down: Some(2),
            nb_initialising: Some(3),
            nb_unmonitored: Some(4),
            total: Some(94),
            percent_up: Some(100.0),
        };
        assert_eq!(parse_monit_report(report), parsed_report);
    }
    #[test]
    fn test_reqwest_error() {
        let error = "error reqwesting http://6.6.6.6:666/_report";
        let parsed_report = MonitReport {
            nb_up: None,
            nb_down: None,
            nb_initialising: None,
            nb_unmonitored: None,
            total: None,
            percent_up: None,
        };
        assert_eq!(parse_monit_report(error), parsed_report);
    }
    #[test]
    fn test_parse_error() {
        // percent is not a percent here, do not panic !
        let report = "up:            85 (toto%)
down:           2 (0.0%)
initialising:   3 (0.0%)
unmonitored:    4 (0.0%)
total:         94 services";
        let parsed_report = MonitReport {
            nb_up: Some(85),
            nb_down: Some(2),
            nb_initialising: Some(3),
            nb_unmonitored: Some(4),
            total: Some(94),
            percent_up: None,
        };
        assert_eq!(parse_monit_report(report), parsed_report);
    }
}
