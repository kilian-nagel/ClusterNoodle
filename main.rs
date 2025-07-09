use std::io;

mod docker {
    pub mod cluster;
    pub mod config;
    pub mod services;
}

use docker::{cluster, config, services};

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

    let nodes_number = nodes_number_input_result.unwrap();
    let config = config::ClusterConfig {
        nodes_number: nodes_number,
    };

    println!("Deploying {nodes_number} machines...");

    if cluster::check_existing_cluster() {
        cluster::destroy_cluster();
        cluster::init_cluster(&config);
        services::create_docker_config_file(&config);
    } else {
        cluster::init_cluster(&config);
        services::create_docker_config_file(&config);
    }
}
