mod docker {
    pub mod cluster;
}

mod utils {
    pub mod command;
    pub mod envVariables;
    pub mod fs;
    pub mod ssh;
}

mod services {
    pub mod services;
}

mod config {
    pub mod config;
}

use crate::config::config::{
    ClusterConfig, build_cluster_nodes_objects, check_conf_file_exists, init_app_config_folder,
};
use crate::services::services::{
    DatabaseType, ServerType, Service, generate_docker_compose, generate_docker_file,
};
use clap::{Args, Parser, Subcommand};
use docker::cluster;
use std::path::PathBuf;
use utils::envVariables::envVariables;
use utils::fs;
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

        #[arg(short)]
        docker_compose_file: Option<String>,
    },
    Stop {},
}

#[derive(Args)]
struct Services {
    #[arg(long, value_enum)]
    server: Option<ServerType>,

    #[arg(long, value_enum)]
    database: Option<DatabaseType>,

    #[arg(long)]
    traefik: bool,
}

fn main() {
    println!("ClusterNoodle v0.1.0");

    // Initialiser les ressources nécessaires pour l'application
    init_app_config_folder();
    check_conf_file_exists();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start {
            docker_compose_file,
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

            // On récupère la configuration des nodes du cluster dans le fichier
            // conf.cluster_noodle.
            let env = envVariables {};
            let conf_file_path = env.get_conf_file_path();
            let nodes_configs = build_cluster_nodes_objects(&conf_file_path);

            if nodes_configs.len() == 0 {
                println!("No nodes config file found (~/.config/conf.cluster_noodle)");
            }

            let mut config = ClusterConfig {
                nodes_number: *nodes_number,
                nodes_configs: nodes_configs,
                cluster_docker_command: String::from(""),
                services: crate::services::services::Services {
                    server: services.server.clone(),
                    database: services.database.clone(),
                    traefik: services.traefik.clone(),
                },
            };

            generate_docker_file(&config);

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
            cluster::deploy_services(docker_compose_file.as_deref());
        }
        Some(Commands::Stop {}) => {
            let mut config = ClusterConfig {
                nodes_number: 0,
                nodes_configs: build_cluster_nodes_objects("conf.cluster_noodle"),
                cluster_docker_command: String::from(""),
                services: services::services::Services {
                    database: None,
                    server: None,
                    traefik: false,
                },
            };

            println!("Stopping the cluster...");
            config.leave_cluster();
            cluster::destroy_cluster();
        }
        None => {}
    }
}
