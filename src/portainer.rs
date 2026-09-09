use reqwest::blocking::Client;
use reqwest::{self, Url};
use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Endpoints {
    #[serde(rename = "AMTDeviceGUID", skip)]
    pub amt_device_guid: u8,
    #[serde(rename = "Agent", skip)]
    pub agent: u8,
    #[serde(rename = "AuthorizedTeams", skip)]
    pub authorized_teams: u8,
    #[serde(rename = "AuthorizedUsers", skip)]
    pub authorized_users: u8,
    #[serde(rename = "AzureCredentials", skip)]
    pub azure_credentials: u8,
    #[serde(rename = "ComposeSyntaxMaxVersion", skip)]
    pub compose_syntax_max_version: u8,
    #[serde(rename = "ContainerEngine", skip)]
    pub container_engine: u8,
    #[serde(rename = "Edge", skip)]
    pub edge: u8,
    #[serde(rename = "EdgeCheckinInterval", skip)]
    pub edge_checkin_interval: u8,
    #[serde(rename = "EdgeID", skip)]
    pub edge_i_d: u8,
    #[serde(rename = "EdgeKey", skip)]
    pub edge_key: u8,
    #[serde(rename = "EnableGPUManagement", skip)]
    pub enable_gpu_management: u8,
    #[serde(rename = "Gpus", skip)]
    pub gpus: u8,
    #[serde(rename = "GroupId", skip)]
    pub group_id: u8,
    #[serde(rename = "Heartbeat", skip)]
    pub heartbeat: u8,
    #[serde(rename = "Id")]
    pub id: i64,
    #[serde(rename = "IsEdgeDevice", skip)]
    pub is_edge_device: u8,
    #[serde(rename = "Kubernetes", skip)]
    pub kubernetes: u8,
    #[serde(rename = "LastCheckInDate", skip)]
    pub last_check_in_date: u8,
    #[serde(rename = "Name", skip)]
    pub name: u8,
    #[serde(rename = "PostInitMigrations", skip)]
    pub post_init_migrations: u8,
    #[serde(rename = "PublicURL")]
    pub public_url: String,
    #[serde(rename = "SecuritySettings", skip)]
    pub security_settings: u8,
    #[serde(rename = "Snapshots", skip)]
    pub snapshots: u8,
    #[serde(rename = "Status", skip)]
    pub status: u8,
    #[serde(rename = "TLS", skip)]
    pub tls: u8,
    #[serde(rename = "TLSCACert", skip)]
    pub tls_ca_cert: u8,
    #[serde(rename = "TLSCert", skip)]
    pub tls_cert: u8,
    #[serde(rename = "TLSConfig", skip)]
    pub tls_config: u8,
    #[serde(rename = "TLSKey", skip)]
    pub tls_key: u8,
    #[serde(rename = "TagIds", skip)]
    pub tag_ids: u8,
    #[serde(rename = "Tags", skip)]
    pub tags: u8,
    #[serde(rename = "TeamAccessPolicies", skip)]
    pub team_access_policies: u8,
    #[serde(rename = "Type", skip)]
    pub endpoint_type: i64,
    #[serde(rename = "URL", skip)]
    pub url: u8,
    #[serde(rename = "UserAccessPolicies", skip)]
    pub user_access_policies: u8,
    #[serde(rename = "UserTrusted", skip)]
    pub user_trusted: u8,
}

#[derive(Debug, Deserialize, Default)]
pub struct Containers {
    #[serde(rename = "Command", skip)]
    pub command: u8,
    #[serde(rename = "Created")]
    pub created: i64,
    #[serde(rename = "HostConfig", skip)]
    pub host_config: u8,
    #[serde(rename = "Id", skip)]
    pub id: u8,
    #[serde(rename = "Image", skip)]
    pub image: u8,
    #[serde(rename = "ImageID", skip)]
    pub image_i_d: u8,
    #[serde(rename = "Labels", skip)]
    pub labels: u8,
    #[serde(rename = "Mounts", skip)]
    pub mounts: u8,
    #[serde(rename = "Names")]
    pub names: Vec<String>,
    #[serde(rename = "NetworkSettings", skip)]
    pub network_settings: u8,
    #[serde(rename = "Ports")]
    pub ports: Vec<Ports>,
    #[serde(rename = "State", skip)]
    pub state: u8,
    #[serde(rename = "Status", skip)]
    pub status: u8,
    #[serde(rename = "SizeRw", skip)]
    pub size_rw: u8,
    #[serde(rename = "SizeRootFs", skip)]
    pub size_root_fs: u8,
}

#[derive(Debug, Deserialize)]
pub struct Ports {
    #[serde(rename = "IP", skip)]
    pub i_p: u8,
    #[serde(rename = "PrivatePort", skip)]
    pub private_port: u8,
    #[serde(rename = "PublicPort")]
    pub public_port: Option<i64>,
    #[serde(rename = "Type", skip)]
    pub port_type: u8,
}

pub async fn get_portainer(
    link: &str,
    auth: String,
) -> Result<Vec<(String, String)>, Box<dyn std::error::Error>> {
    let api_base = Url::parse(link)?;

    let endpoint_list = api_base.join("endpoints")?;
    let client = Client::builder()
        .tls_danger_accept_invalid_certs(true)
        .build()
        .expect("failed to build client");
    let response = client
        .get(endpoint_list.clone())
        .header("X-API-KEY", &auth)
        .send()?;
    let endpoints: Vec<Endpoints> = response.json()?;

    let mut ports: Vec<(String, String)> = Vec::new();
    for endpoint in endpoints.iter() {
        if endpoint.public_url.is_empty() || endpoint.id == 7 {
            continue;
        }

        let mut endpoint_url = Url::parse(&format!("http://{}", endpoint.public_url))?;

        let containers_api = endpoint_list.join(&format!(
            "endpoints/{}/docker/containers/json",
            endpoint.id
        ))?;

        let response = client
            .get(containers_api)
            .header("X-API-KEY", &auth)
            .send()?;

        let containers: Vec<Containers> = response.json()?;
        for container in containers {
            for port in container.ports {
                if let Some(port_num) = port.public_port {
                    endpoint_url
                        .set_port(Some(port_num.try_into().expect("setting port failed")))
                        .map_err(|_| "cannot be base")?;

                    let cont_name = container
                        .names
                        .first()
                        .expect("Unnamed container?")
                        .to_owned();
                    let clean_name = cont_name.strip_prefix("/").expect("string").to_string();
                    ports.push((clean_name, endpoint_url.to_string()));
                }
            }
        }
    }
    Ok(ports)
}
