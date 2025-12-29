#![allow(non_snake_case)]
mod docker {
    pub mod cluster;
}

mod utils {
    pub mod command;
    pub mod envParsing;
    pub mod envVariables;
    pub mod fs;
    pub mod ssh;
}

mod models {
    pub mod network;
    pub mod service;
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
        #[command(flatten)]
        services: Services,

        #[arg(short)]
        ip_adress: Option<String>,

        #[arg(short)]
        docker_compose_file: Option<String>,

        #[arg(short, long)]
        no_rebuild_docker_compose_file: bool,

        #[arg(long)]
        project_folder_path: Option<String>,

        #[arg(short)]
        project_entry_file_path: Option<String>,
        
        #[arg(long)]
        ssl_certificate_path_key: Option<String>,

        #[arg(long)]
        ssl_certificate_path_crt: Option<String>,
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

    #[arg(long)]
    dashboard: bool,
}

fn main() {
    const VERSION: &str = env!("CARGO_PKG_VERSION");
    println!("ClusterNoodle {}", VERSION);

    // Initialiser les ressources nécessaires pour l'application
    init_app_config_folder();
    check_conf_file_exists();

    let cli = Cli::parse();

    match &cli.command {
        Some(Commands::Start {
            docker_compose_file,
            services,
            ip_adress,
            project_folder_path,
            project_entry_file_path,
            ssl_certificate_path_key,
            ssl_certificate_path_crt,
            no_rebuild_docker_compose_file
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

            let mut config = ClusterConfig {
                nodes_number: 0,
                nodes_configs: nodes_configs,
                cluster_docker_command: String::from(""),
                services: crate::services::services::Services {
                    server: services.server.clone(),
                    database: services.database.clone(),
                    traefik: services.traefik.clone(),
                    dashboard: services.dashboard.clone()
                },
                ip_adress: ip_adress.clone(),
                project_folder_path: project_folder_path.clone(),
                project_entry_file_path: project_entry_file_path.clone(),
                ssl_certificate_path_key: ssl_certificate_path_key.clone(),
                ssl_certificate_path_crt: ssl_certificate_path_crt.clone(),
                docker_images: vec![]
            };

            // Fetch and set IP address before generating docker-compose file
            println!("Fetching IP address...");
            config.fetch_and_set_ip_address(ip_adress);

            // On ne génère le fichier doccaptker_compose uniquement si l'utilisateur n'a pas renseigné
            // le sien.
            if !docker_compose_file.is_some() && !*no_rebuild_docker_compose_file  {
                println!("Generating docker-compose file...");
                if let Err(e) = generate_docker_file(&mut config) {
                    eprintln!("Error generating docker-compose file: {}", e);
                }
            }

            // Init le cluster
            println!("Intializing cluster...");
            config.init_cluster();

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

            println!("Pulling docker images... This may take a while.");
            config.pull_docker_images();

            // Déploiement des services docker
            println!("Deploying services to the cluster...");
            cluster::deploy_services();
        }
        Some(Commands::Stop {}) => {
            let env = EnvVariables {};
            let conf_file_path = env.get_conf_file_path();

            let mut config = ClusterConfig {
                ip_adress: Some(String::from("")),
                nodes_number: 0,
                nodes_configs: build_cluster_nodes_objects(&conf_file_path),
                cluster_docker_command: String::from(""),
                services: services::services::Services {
                    database: None,
                    server: None,
                    traefik: false,
                    dashboard: false
                },
                project_folder_path: Some(String::from("")),
                project_entry_file_path: Some(String::from("")),
                ssl_certificate_path_key: Some(String::from("")),
                ssl_certificate_path_crt: Some(String::from("")),
                docker_images: vec![]
            };

            println!("Stopping the cluster...");
            config.leave_cluster();
            cluster::destroy_cluster();
        }
        None => {}
    }
}
