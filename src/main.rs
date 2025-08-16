mod docker {
    pub mod cluster;
}

mod utils {
    pub mod command;
    pub mod ssh;
}

mod config {
    pub mod config;
}

use crate::config::config::{ClusterConfig, Service, build_cluster_nodes_objects};
use clap::{Args, Parser, Subcommand};
use docker::cluster;
use std::path::PathBuf;
use utils::ssh;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    name: Option<String>,

    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    #[arg(short, long, action = clap::ArgAction::Count)]
    debug: u8,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Start {
        #[arg(short)]
        nodes_number: u16,

        #[command(flatten)]
        services: Services,
    },
    Stop {},
}

#[derive(Args)]
#[group(required = true, multiple = true)]
struct Services {
    #[arg(long)]
    server: bool,

    #[arg(long)]
    database: bool,

    #[arg(long)]
    traefik: bool,
}

fn main() {
    println!("ClusterNoodle v0.1.0");
    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start {
            nodes_number,
            services,
        }) => {
            // Vérifie s'il y a déjà un cluster en exécution
            if cluster::check_existing_cluster() {
                println!(
                    "A cluster is already running. You have to stop the running cluster before starting a new one."
                );
                return;
            }

            // Sélection des services à déployer
            let mut services_config = vec![];
            if services.server {
                services_config.push(Service::Server);
            };
            if services.database {
                services_config.push(Service::Database);
            };
            if services.traefik {
                services_config.push(Service::Traefik);
            };

            // On récupère la configuration des nodes du cluster dans le fichier
            // conf.cluster_noodle.
            let nodes_configs = build_cluster_nodes_objects("conf.cluster_noodle");
            if nodes_configs.len() == 0 {
                println!("No nodes config file found (conf.cluster_noodle)");
            }

            let mut config = ClusterConfig {
                nodes_number: *nodes_number,
                nodes_configs: nodes_configs,
                cluster_docker_command: String::from(""),
                services: services_config,
            };

            // On met à jour le fichier de config en fonction des services sélectionnées
            config.update_docker_config_file();

            // Init le cluster
            config.init();

            // Création des clés et connexions en SSH aux nodes
            if !ssh::check_existing_ssh_key() {
                ssh::generate_ssh_key();
            }
            ssh::copy_ssh_key_to_machines(&config);

            // Installation de docker sur chaque machine
            config.install_docker();

            // On fait rejoindre le cluster à chaque machine
            config.join_cluster();

            // Déploiement des services docker
            cluster::deploy_services();
        }
        Some(Commands::Stop {}) => {
            let mut config = ClusterConfig {
                nodes_number: 0,
                nodes_configs: build_cluster_nodes_objects("conf.cluster_noodle"),
                cluster_docker_command: String::from(""),
                services: vec![],
            };

            println!("Stopping the cluster...");
            config.leave_cluster();
            cluster::destroy_cluster();
        }
        None => {}
    }
}
