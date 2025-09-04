use crate::fs::path_exists;
use crate::services::services::Services;
use crate::utils::envVariables::envVariables;
use std::fs;

pub struct NodeConfig {
    pub ip: String,
    pub username: String,
    pub password: String,
}

pub struct ClusterConfig {
    pub nodes_number: u16,
    pub nodes_configs: Vec<NodeConfig>,
    pub cluster_docker_command: String,
    pub services: Services,
}

pub fn init_app_config_folder() {
    let env = envVariables {};
    let config_folder_path = env.get_conf_path();
    fs::create_dir_all(config_folder_path);
}

pub fn check_conf_file_exists() {
    let env = envVariables {};
    let conf_file_path = env.get_conf_file_path();

    match path_exists(&conf_file_path) {
        Ok(v) => (),
        Err(e) => {
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
        }
    }

    return nodes_configs;
}
