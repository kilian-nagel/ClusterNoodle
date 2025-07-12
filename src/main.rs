use std::io;

mod docker {
    pub mod cluster;
    pub mod config;
    pub mod services;
}

mod utils {
    pub mod command;
    pub mod ssh;
}

use docker::{cluster, config, services};
use utils::ssh;

fn main() {
    println!("ClusterNoodle v0.1.0");

    let nodes_number = cluster::ask_nodes_number();
    let nodes_configs = cluster::ask_nodes_infos();

    let config = config::ClusterConfig {
        nodes_number: nodes_number,
        nodes_configs: nodes_configs,
    };

    println!("Deploying cluster...");

    if cluster::check_existing_cluster() {
        cluster::destroy_cluster();
    }

    cluster::init_cluster();
    services::create_docker_config_file(&config);

    if !ssh::check_existing_ssh_key() {
        ssh::generate_ssh_key();
    }

    ssh::copy_ssh_key_to_machines(&config);
}
