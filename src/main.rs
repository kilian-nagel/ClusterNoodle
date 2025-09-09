#![allow(non_snake_case)]
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
    pub mod apache {
        pub mod apache;
    }
    pub mod nginx {
        pub mod nginx;
    }
}

mod config {
    pub mod config;
}

use crate::config::config::{
    ClusterConfig, build_cluster_nodes_objects, check_conf_file_exists, init_app_config_folder,
};
use crate::services::services::{DatabaseType, ServerType, generate_docker_file};
use clap::{Args, Parser, Subcommand};
use docker::cluster;
use std::path::PathBuf;
use utils::envVariables::EnvVariables;
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

        #[arg(long)]
        project_folder_path: String,

        #[arg(short)]
        project_entry_file_path: Option<String>,
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
            project_folder_path,
            project_entry_file_path
        }) => {
            let services_specified =
                services.server.is_some() || services.database.is_some() || services.traefik;

            if docker_compose_file.is_some() && services_specified {
                eprintln!(
                    "Error: You cannot specify both `--docker-compose-file` and individual services."
                );
                std::process::exit(1);
            }

            if docker_compose_file.is_none() && !services_specified {
                eprintln!(
                    "Error: You must specify either `--docker-compose-file` or at least one service."
                );
                std::process::exit(1);
            }

            // Vérifie s'il y a déjà un cluster en exécution
            if cluster::check_existing_cluster() {
                println!(
                    "A cluster is already running. You have to stop the running cluster before starting a new one."
                );
                return;
            }

            // On récupère la configuration des nodes du cluster dans le fichier
            // conf.cluster_noodle.

            println!("Getting servers from the conf file...");
            let env = EnvVariables {};
            let conf_file_path = env.get_conf_file_path();
            let nodes_configs = build_cluster_nodes_objects(&conf_file_path);

            if nodes_configs.len() == 0 {
                println!("No nodes config file found (~/.config/ClusterNoodle/conf.cluster_noodle)");
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
                project_folder_path: project_folder_path.to_string(),
                project_entry_file_path: project_entry_file_path.clone()
            };

            // On ne génère le fichier doccaptker_compose uniquement si l'utilisateur n'a pas renseigné
            // le sien.
            if !docker_compose_file.is_some() {
                println!("Generating docker-compose file...");
                if let Err(e) = generate_docker_file(&config) {
                    eprintln!("Error generating docker-compose file: {}", e);
                }
            }

            // Init le cluster
            println!("Intializing cluster...");
            config.init();

            // Création des clés et connexions en SSH aux nodes
            println!("Generating ssh keys...");
            if !ssh::check_existing_ssh_key() {
                ssh::generate_ssh_key();
            }
            ssh::copy_ssh_key_to_machines(&config);

            // Installation de docker sur chaque machine

            println!("Installating docker on target servers...");
            config.install_docker();

            // On fait rejoindre le cluster à chaque machine
            println!("Target servers are joining the cluster...");
            config.join_cluster();

            // Déploiement des services docker
            println!("Deploying services to the cluster...");
            cluster::deploy_services(docker_compose_file.as_deref());
        }
        Some(Commands::Stop {}) => {
            let env = EnvVariables {};
            let conf_file_path = env.get_conf_file_path();

            let mut config = ClusterConfig {
                nodes_number: 0,
                nodes_configs: build_cluster_nodes_objects(&conf_file_path),
                cluster_docker_command: String::from(""),
                services: services::services::Services {
                    database: None,
                    server: None,
                    traefik: false,
                },
                project_folder_path: String::from(""),
                project_entry_file_path: Some(String::from("")),
            };

            println!("Stopping the cluster...");
            config.leave_cluster();
            cluster::destroy_cluster();
        }
        None => {}
    }
}
