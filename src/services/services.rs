use crate::ClusterConfig;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io;
use std::path::PathBuf; // Re-export at module level

#[derive(Debug, Clone, PartialEq)]
pub enum Service {
    Server,
    Database,
    Traefik,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ServerType {
    Nginx,
    Apache,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum DatabaseType {
    #[value(name = "mysql")]
    MySQL,
    #[value(name = "postgresql")]
    PostgreSQL,
    #[value(name = "mongodb")]
    MongoDB,
}

pub struct Services {
    pub server: Option<ServerType>,
    pub database: Option<DatabaseType>,
    pub traefik: bool,
}

#[derive(Serialize, Deserialize)]
struct DockerComposeService {
    image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ports: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    environment: Option<HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    labels: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    depends_on: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct DockerCompose {
    version: String,
    services: HashMap<String, DockerComposeService>,
    #[serde(skip_serializing_if = "Option::is_none")]
    volumes: Option<HashMap<String, serde_yaml::Value>>,
}

pub fn generate_docker_compose(
    cluster_config: &ClusterConfig,
) -> Result<String, Box<dyn std::error::Error>> {
    let mut compose = DockerCompose {
        version: "3.9".to_string(),
        services: HashMap::new(),
        volumes: None,
    };

    let mut volumes = HashMap::new();

    if cluster_config.services.traefik {
        add_traefik_service(&mut compose);
    }

    if let Some(server) = &cluster_config.services.server {
        add_server_service(&mut compose, server);
    }

    if let Some(database) = &cluster_config.services.database {
        let mut volumes: HashMap<String, serde_yaml::Value> = HashMap::new();
        add_database_service(&mut compose, &database, &mut volumes);
    }

    // Ajoute les volumes s'il y en avait
    if !volumes.is_empty() {
        compose.volumes = Some(volumes);
    }

    // Convertir le tout en yaml
    let yaml = serde_yaml::to_string(&compose)?;
    Ok(yaml)
}

fn add_traefik_service(compose: &mut DockerCompose) {
    let traefik_service = DockerComposeService {
        image: "traefik:v3.0".to_string(),
        command: Some(vec![
            "--api.dashboard=true".to_string(),
            "--providers.docker=true".to_string(),
            "--entrypoints.web.address=:80".to_string(),
            "--metrics.prometheus=true".to_string(),
        ]),
        ports: Some(vec!["80:80".to_string(), "8080:8080".to_string()]),
        volumes: Some(vec![
            "/var/run/docker.sock:/var/run/docker.sock:ro".to_string(),
        ]),
        labels: Some(vec![
            "traefik.enable=true".to_string(),
            "traefik.http.routers.traefik.rule=Host(`traefik.localhost`)".to_string(),
            "traefik.http.routers.traefik.service=api@internal".to_string(),
        ]),
        environment: None,
        depends_on: None,
    };

    compose
        .services
        .insert("traefik".to_string(), traefik_service);
}

fn add_server_service(compose: &mut DockerCompose, server_type: &ServerType) {
    match server_type {
        ServerType::Nginx => {
            let nginx_service = DockerComposeService {
                image: "nginx:latest".to_string(),
                labels: Some(vec![
                    "traefik.enable=true".to_string(),
                    "traefik.http.routers.nginx.rule=Host(`nginx.localhost`)".to_string(),
                    "traefik.http.services.nginx.loadbalancer.server.port=80".to_string(),
                ]),
                command: None,
                ports: None,
                volumes: None,
                environment: None,
                depends_on: None,
            };
            compose.services.insert("nginx".to_string(), nginx_service);
        }
        ServerType::Apache => {
            let apache_service = DockerComposeService {
                image: "httpd:latest".to_string(),
                labels: Some(vec![
                    "traefik.enable=true".to_string(),
                    "traefik.http.routers.apache.rule=Host(`apache.localhost`)".to_string(),
                    "traefik.http.services.apache.loadbalancer.server.port=80".to_string(),
                ]),
                command: None,
                ports: None,
                volumes: None,
                environment: None,
                depends_on: None,
            };
            compose
                .services
                .insert("apache".to_string(), apache_service);
        }
    }
}

fn add_database_service(
    compose: &mut DockerCompose,
    database_type: &DatabaseType,
    volumes: &mut HashMap<String, serde_yaml::Value>,
) {
    match database_type {
        DatabaseType::MySQL => {
            let mut mysql_env = HashMap::new();
            mysql_env.insert(
                "MYSQL_ROOT_PASSWORD".to_string(),
                "rootpassword".to_string(),
            );
            mysql_env.insert("MYSQL_DATABASE".to_string(), "app".to_string());
            mysql_env.insert("MYSQL_USER".to_string(), "appuser".to_string());
            mysql_env.insert("MYSQL_PASSWORD".to_string(), "apppassword".to_string());

            let mysql_service = DockerComposeService {
                image: "mysql:8.4".to_string(),
                environment: Some(mysql_env),
                ports: Some(vec!["3306:3306".to_string()]),
                volumes: Some(vec!["mysql_data:/var/lib/mysql".to_string()]),
                command: None,
                labels: None,
                depends_on: None,
            };

            compose.services.insert("mysql".to_string(), mysql_service);
            volumes.insert("mysql_data".to_string(), serde_yaml::Value::Null);

            // Add MySQL exporter
            let mut exporter_env = HashMap::new();
            exporter_env.insert(
                "DATA_SOURCE_NAME".to_string(),
                "appuser:apppassword@(mysql:3306)/app".to_string(),
            );

            let mysqld_exporter = DockerComposeService {
                image: "prom/mysqld-exporter:latest".to_string(),
                environment: Some(exporter_env),
                depends_on: Some(vec!["mysql".to_string()]),
                labels: Some(vec![
                    "traefik.enable=true".to_string(),
                    "traefik.http.routers.mysqlmetrics.rule=Host(`mysql-metrics.localhost`)"
                        .to_string(),
                    "traefik.http.services.mysqlmetrics.loadbalancer.server.port=9104".to_string(),
                ]),
                command: None,
                ports: None,
                volumes: None,
            };

            compose
                .services
                .insert("mysqld_exporter".to_string(), mysqld_exporter);
        }
        DatabaseType::PostgreSQL => {
            let mut postgres_env = HashMap::new();
            postgres_env.insert("POSTGRES_DB".to_string(), "app".to_string());
            postgres_env.insert("POSTGRES_USER".to_string(), "appuser".to_string());
            postgres_env.insert("POSTGRES_PASSWORD".to_string(), "apppassword".to_string());

            let postgres_service = DockerComposeService {
                image: "postgres:15".to_string(),
                environment: Some(postgres_env),
                ports: Some(vec!["5432:5432".to_string()]),
                volumes: Some(vec!["postgres_data:/var/lib/postgresql/data".to_string()]),
                command: None,
                labels: None,
                depends_on: None,
            };

            compose
                .services
                .insert("postgres".to_string(), postgres_service);
            volumes.insert("postgres_data".to_string(), serde_yaml::Value::Null);
        }
        DatabaseType::MongoDB => {
            let mut mongo_env = HashMap::new();
            mongo_env.insert(
                "MONGO_INITDB_ROOT_USERNAME".to_string(),
                "admin".to_string(),
            );
            mongo_env.insert(
                "MONGO_INITDB_ROOT_PASSWORD".to_string(),
                "adminpassword".to_string(),
            );

            let mongo_service = DockerComposeService {
                image: "mongo:7".to_string(),
                environment: Some(mongo_env),
                ports: Some(vec!["27017:27017".to_string()]),
                volumes: Some(vec!["mongo_data:/data/db".to_string()]),
                command: None,
                labels: None,
                depends_on: None,
            };

            compose.services.insert("mongo".to_string(), mongo_service);
            volumes.insert("mongo_data".to_string(), serde_yaml::Value::Null);
        }
    }
}

pub fn create_docker_file(dockerfile_content: &str) -> io::Result<()> {
    fs::write("./src/docker/docker-compose.yml", dockerfile_content)?;
    Ok(())
}
