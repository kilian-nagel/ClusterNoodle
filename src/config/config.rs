#![allow(dead_code)]
use crate::fs::path_exists;
use crate::services::services::Services;
use crate::utils::envVariables::EnvVariables;
use std::fs;

pub struct NodeConfig {
    pub ip: String,
    pub username: String,
    pub password: String,
}

pub struct ClusterConfig {
    pub ip_adress: Option<String>,
    pub nodes_number: u16,
    pub nodes_configs: Vec<NodeConfig>,
    pub cluster_docker_command: String,
    pub project_folder_path: Option<String>,
    pub project_entry_file_path: Option<String>,
    pub ssl_certificate_path_key: Option<String>,
    pub ssl_certificate_path_crt: Option<String>,
    pub services: Services,
    pub docker_images: Vec<String>
}

pub fn init_app_config_folder() {
    let env = EnvVariables {};
    dotenvy::from_path(&env.get_env_file_path()).unwrap();
}

pub fn check_conf_file_exists() {
    let env = EnvVariables {};
    let conf_file_path = env.get_conf_file_path();

    match path_exists(&conf_file_path) {
        Ok(_v) => (),
        Err(_e) => {
            panic!(
                "No conf file found. You need to declare servers in the conf file (~/.config/conf.cluster_noodle)"
            );
        }
    }
}

pub fn build_cluster_nodes_objects(file_path: &str) -> Vec<NodeConfig> {
    let contents = fs::read_to_string(file_path)
        .expect("Failed to read config file (conf.cluster_noodle). \n");

    let lines: Vec<&str> = contents.lines().collect();
    let mut nodes_configs: Vec<NodeConfig> = vec![];

    for line in lines {
        let data: Vec<&str> = line.split(",").collect::<Vec<&str>>();
        if data.len() > 2 {
            let node_config = NodeConfig {
                ip: data[0].to_string(),
                username: data[1].to_string(),
                password: data[2].to_string(),
            };
            nodes_configs.push(node_config);
        } else if data.len() > 1 {
            let node_config = NodeConfig {
                ip: data[0].to_string(),
                username: data[1].to_string(),
                password: String::from(""),
            };
            nodes_configs.push(node_config);
        } else if data.len() == 1 {
            panic!("For each server declared in the conf file you need to provide at least ip,username");
        }
    }

    return nodes_configs;
}
