use kuma_client::{Client, Config, Url, monitor::MonitorHttp};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs::{File, write};
use std::io::{BufReader, Write};
use std::process::Command;
use u_got_miau::portainer;
use u_got_miau::secret_parser::*;
use yaml_serde;
use curl_http_client::*;
use http::{Method, Request};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Port {
    pub abbr: String,
    pub href: String,
}

#[tokio::main()]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let secret = read_secret();
    let api_link = secret.p_addr.clone();

    let mut a = portainer::get_portainer(&api_link, secret.p_auth.clone()).await?;
    a.dedup();

    let kuma = Client::connect(Config {
        url: Url::parse(&secret.k_addr).expect("Invalid URL"),
        username: Some(secret.k_un.clone()),
        password: Some(secret.k_pw.clone()),
        ..Default::default()
    })
    .await
    .expect("Failed to connect to server");

    let existing_monitors = &mut kuma
        .get_monitors()
        .await
        .expect("failed to fetch list of existing monitors")
        .iter()
        .map(|(_, v)| match v {
            kuma_client::models::monitor::Monitor::Keyword { value } => value.url.clone().unwrap(),
            kuma_client::models::monitor::Monitor::Http { value } => value.url.clone().unwrap(),
            _ => "".to_string(),
        })
        .collect::<Vec<String>>();

    existing_monitors.dedup();

    let mut saved_entries = Vec::new();

    for (name, address) in a {
        let collector = Collector::Ram(Vec::new());

        let request = Request::builder()
            .uri(address)
            .method(Method::GET)
            .body(None)
            .unwrap();

        let response = HttpClient::new(collector)
            .request(request).unwrap()
            .blocking()
            .perform();

        if !existing_monitors.contains(&address) || response.is_ok() {
            to_add_to_homepage(&name, &address, &mut saved_entries);
            add_to_kuma(&kuma, &name, address).await;
        }
    }

    let _ = homepage_sorter(saved_entries);

    Ok(())
}

async fn add_to_kuma(kuma: &Client, name: &String, address: String) {
    kuma.add_monitor(MonitorHttp {
        name: Some(name.to_string()),
        url: Some(address),
        timeout: Some(60),
        max_retries: Some(3),
        resend_interval: Some(120),
        ..Default::default()
    })
    .await
    .expect("Failed to add monitor");
}



fn to_add_to_homepage(name: &String, address: &str, saved_entries: &mut Vec<String>) {
    let mut abbr = name.clone();
    let _ = abbr.split_off(2);

    let mut inst_counter: u8 = 0;
    for entry in saved_entries.iter() {
        if entry.contains(name) {
            inst_counter += 1;
        }
    }
    
    let mut mod_name = name.clone();
    let mut icon = format!("abbr: {abbr}");

    if inst_counter != 0 {
        mod_name.push_str(&format!("{inst_counter}"));
    }
    
    let collector = Collector::Ram(Vec::new());

    let request = Request::builder()
        .uri(format!("https://cdn.jsdelivr.net/gh/homarr-labs/dashboard-icons/svg/{mod_name}.svg"))
        .method(Method::GET)
        .body(None)
        .unwrap();

    let response = HttpClient::new(collector)
        .request(request).unwrap()
        .blocking()
        .perform()
        .unwrap();

    let resp = response.headers().get("content-type").expect("cant get type").to_str().expect("parse failed");

    if resp.contains("image")  {
        icon = format!("icon: {mod_name}");
    }

    let data = format!(
        "\n  - {mod_name}:
    - {icon}
      href: {address}"
    );
    saved_entries.push(data)
}

pub fn homepage_sorter(saved_entries: Vec<String>) {
    let secret = read_secret();

    let mut filepath = secret.neutrino.clone();
    filepath.push_str(":");
    filepath.push_str(&secret.hp_path);

    let ssh_handler = Command::new("scp")
        .args(["-i", &secret.key, &filepath, "."])
        .spawn()
        .expect("scp failed");

    let _ = ssh_handler.wait_with_output();

    let mut appender = File::options()
        .append(true)
        .open("bookmarks.yaml")
        .expect("can't open file");
    for entry in saved_entries {
        writeln!(&mut appender, "{}", entry).expect("cant write homepage entry");
    }

    let reader = File::open("bookmarks.yaml").expect("can't open file");
    let buf_reader = BufReader::new(reader);
    let raw_read: Vec<HashMap<String, Vec<HashMap<String, Vec<Port>>>>> =
        yaml_serde::from_reader(buf_reader).expect("can't read file");

    let contents = raw_read
        .first()
        .expect("empty")
        .iter()
        .next()
        .expect("contents not found")
        .1;

    let mut new_mapping: BTreeMap<String, Port> = BTreeMap::new();

    for port in contents {
        let entry = port.iter().next().expect("can't find port");
        let key = entry.0.clone();
        let value = entry.1.first().expect("port data lost").clone();
        new_mapping.insert(key, value);
    }

    let mut new_contents = Vec::new();

    for port in new_mapping {
        let entry = HashMap::from([(port.0, vec![port.1])]);
        new_contents.push(entry);
    }

    let final_write: Vec<HashMap<String, Vec<HashMap<String, Vec<Port>>>>> =
        vec![HashMap::from([("Ports".to_string(), new_contents)])];

    let yaml = yaml_serde::to_string(&final_write).expect("can't yaml it");

    let _ = write("bookmarks.yaml", yaml);

    let filepath = filepath.split("b").next().expect("filepath empty?");

    let ssh_handler = Command::new("scp")
        .args(["-i", &secret.key, "bookmarks.yaml", &filepath])
        .spawn()
        .expect("scp failed");

    let _ = ssh_handler.wait_with_output();
}
