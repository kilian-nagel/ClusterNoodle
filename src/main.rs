use std::io;

mod docker {
    pub mod cluster;
    pub mod services;
}

mod utils {
    pub mod command;
    pub mod ssh;
}

mod config {
    pub mod config;
}

use crate::config::config::{ClusterConfig, build_cluster_nodes_objects};
use clap::{Parser, Subcommand};
use docker::{cluster, services};
use std::path::PathBuf;
use utils::ssh;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Optional name to operate on
    name: Option<String>,

    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    start {
        #[arg(short)]
        nodes_number: u16,
    },
    stop {},
}

fn main() {
    println!("ClusterNoodle v0.1.0");
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::start { nodes_number }) => {
            if cluster::check_existing_cluster() {
                println!(
                    "A cluster is already running. You have to stop the running cluster before starting a new one."
                );
                return;
            }
            println!("Deploying cluster with {} nodes", nodes_number);

            let nodes_configs = build_cluster_nodes_objects("conf.cluster_noodle");
            if nodes_configs.len() == 0 {
                println!("No nodes config file found (conf.cluster_noodle)");
            }

            let mut config = ClusterConfig {
                nodes_number: *nodes_number,
                nodes_configs: nodes_configs,
                cluster_docker_command: String::from(""),
            };

            cluster::init_cluster(&mut config);
            services::create_docker_config_file(&config);

            if !ssh::check_existing_ssh_key() {
                ssh::generate_ssh_key();
            }

            ssh::copy_ssh_key_to_machines(&config);
            cluster::install_docker(&config);
            cluster::join_cluster(&config);
            cluster::deploy_services(&config);
        }
        Some(Commands::stop {}) => {
            let mut config = ClusterConfig {
                nodes_number: 0,
                nodes_configs: build_cluster_nodes_objects("conf.cluster_noodle"),
                cluster_docker_command: String::from(""),
            };

            println!("Stopping the cluster...");
            cluster::leave_cluster(&config);
            cluster::destroy_cluster();
        }
        None => {}
    }
}
