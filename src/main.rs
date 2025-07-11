use std::io;

mod docker {
    pub mod cluster;
    pub mod config;
    pub mod services;
}

mod utils {
    pub mod ssh;
}

use docker::{cluster, config, services};
use utils::ssh;

fn main() {
    println!("ClusterNoodle v0.1.0");
    println!("How many nodes would like you deploy ?");
    let mut nodes_number_input = String::new();

    io::stdin().read_line(&mut nodes_number_input).unwrap();
    // On se débarasse du \n.
    nodes_number_input.pop();

    let nodes_number_input_result = nodes_number_input.parse::<u16>();
    if !nodes_number_input_result.is_ok() {
        panic!("Number of nodes has to be a number");
    }

    println!(
        "You have to give servers informations. Exemple : 192.168.1.1,username,password 192.168.1.2,username2,password2"
    );
    let mut nodes_ips_input = String::new();

    io::stdin().read_line(&mut nodes_ips_input).unwrap();
    // On se débarasse du \n.
    nodes_ips_input.pop();

    let nodes_ips = nodes_ips_input.split(" ");
    let mut nodes_configs: Vec<config::NodeConfig> = vec![];

    for node_ip in nodes_ips {
        let data = node_ip.split(",").collect::<Vec<_>>();
        if data.len() > 2 {
            let node_config = config::NodeConfig {
                ip: data[0].to_string(),
                username: data[1].to_string(),
                password: data[2].to_string(),
            };
            nodes_configs.push(node_config);
        }
    }

    let nodes_number = nodes_number_input_result.unwrap();
    let config = config::ClusterConfig {
        nodes_number: nodes_number,
        nodes_configs: nodes_configs,
    };

    println!("Deploying cluster...");

    if cluster::check_existing_cluster() {
        cluster::destroy_cluster();
    }

    cluster::init_cluster(&config);
    services::create_docker_config_file(&config);

    if !ssh::check_existing_ssh_key() {
        ssh::generate_ssh_key();
    }

    ssh::copy_ssh_key_to_machines(&config);
}
