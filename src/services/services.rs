#![allow(dead_code)]
use crate::ClusterConfig;
use crate::services::apache::apache::ApacheConfig;
use crate::services::nginx::nginx::NginxConfig;
use crate::utils::envParsing::EnvConfig;
use crate::utils::envVariables::EnvVariables;
use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use std::collections::HashMap;
use std::fs;
use std::io;

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
    NodeJS,
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

struct DockerComposeBuilder<'a> {
    cluster_config: &'a ClusterConfig,
    volumes: HashMap<String, serde_yaml::Value>,
    compose: DockerCompose,
}

impl<'a> DockerComposeBuilder<'a> {
    pub fn generate_docker_compose(&mut self) -> Result<String, Box<dyn std::error::Error>> {
        if self.cluster_config.services.traefik {
            self.add_traefik_service();
        }

        self.add_server_service();
        self.add_database_service();

        // Convertir le tout en yaml
        let yaml = serde_yaml::to_string(&self.compose)?;
        Ok(yaml)
    }

    fn add_traefik_service(&mut self) {
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

        self.compose
            .services
            .insert("traefik".to_string(), traefik_service);
    }

    fn add_server_service(&mut self) {
        match &self.cluster_config.services.server {
            Some(ServerType::Nginx) => {
                // Volume qui concerne la config du serveur Nginx.
                let conf_volume = format!(
                    "{}:/etc/nginx/conf.d/server.conf:ro",
                    NginxConfig::get_config_path()
                );
                
                let mut nginx_service = DockerComposeService {
                    image: "trafex/php-nginx:3.9.0".to_string(),
                    labels: None,   
                    command: None,
                    ports: Some(vec![format!("80:8080")]),
                    volumes: Some(vec![conf_volume]),
                    environment: None,
                    depends_on: None,
                };

                if let Some(ref mut volumes) = nginx_service.volumes && self.cluster_config.project_folder_path.is_some() {
                    let absolute_path = fs::canonicalize(self.cluster_config.project_folder_path.clone().unwrap()).unwrap();
                    let usable_path = absolute_path.to_str().unwrap();
                    // Volume qui concerne le code à exécuter dans le serveur Nginx.
                    volumes.push(format!("{}:/var/www/html",usable_path));
                };

                if let Some(crt_path) = &self.cluster_config.ssl_certificate_path_crt  && let Some(key_path) = &self.cluster_config.ssl_certificate_path_key{
                    let crt_absolute_path = fs::canonicalize(crt_path).unwrap();
                    let key_absolute_path = fs::canonicalize(key_path).unwrap();

                    let crt_path_usable = crt_absolute_path.to_str().unwrap();
                    let key_path_usable = key_absolute_path.to_str().unwrap();

                    if let Some(ref mut vols) = nginx_service.volumes {
                        vols.push(format!("{}:/etc/nginx/ssl/crt.pem:ro", crt_path_usable));
                        vols.push(format!("{}:/etc/nginx/ssl/key.pem:ro", key_path_usable));
                    }

                    if let Some(ref mut ports) = nginx_service.ports {
                        ports.push(format!("443:443"));
                    }
                } else if self.cluster_config.ssl_certificate_path_crt.is_some() && self.cluster_config.ssl_certificate_path_key.is_none() {
                    println!("Seul le certificat TLS a été renseigné il manque la clé (--ssl_certificate_path_key) !");
                } else if self.cluster_config.ssl_certificate_path_key.is_some() && self.cluster_config.ssl_certificate_path_crt.is_none() {
                    println!("Seul la clé TLS a été renseignée il manque le certificat (--ssl_certificate_path_crt)!");
                }

                if self.cluster_config.services.traefik {
                    nginx_service.labels = Some(vec![
                        "traefik.enable=true".to_string(),
                        "traefik.http.routers.nginx.rule=Host(`nginx.localhost`)".to_string(),
                        "traefik.http.services.nginx.loadbalancer.server.port=80".to_string(),
                    ]);
                }
                self.compose
                    .services
                    .insert("nginx".to_string(), nginx_service);
            }

            Some(ServerType::Apache) => {
                // Volume qui concerne la config du serveur Apache.
                let vhost_path_volume = format!(
                    "{}:/opt/docker/etc/httpd/vhost.conf:ro",
                    ApacheConfig::get_vhost_config_path()
                );

                let mut apache_service = DockerComposeService {
                    image: "webdevops/php-apache:8.4".to_string(),
                    labels: None,
                    command: None,
                    ports: Some(vec![format!("8080:80")]),
                    volumes: Some(vec![vhost_path_volume]),
                    environment: None,
                    depends_on: None,
                };  

                if let Some(ref mut volumes) = apache_service.volumes && self.cluster_config.project_folder_path.is_some() {
                    // Volume qui concerne le code à exécuter dans le serveur Apache.
                    let app_path = fs::canonicalize(self.cluster_config.project_folder_path.clone().unwrap()).unwrap();
                    let app_path_usable = app_path.to_str().unwrap();
                    volumes.push(format!("{}:/app", app_path_usable));
                };

                if let Some(crt_path) = &self.cluster_config.ssl_certificate_path_crt  && let Some(key_path) = &self.cluster_config.ssl_certificate_path_key{
                    let crt_absolute_path = fs::canonicalize(crt_path).unwrap();
                    let key_absolute_path = fs::canonicalize(key_path).unwrap();

                    let crt_path_usable = crt_absolute_path.to_str().unwrap();
                    let key_path_usable = key_absolute_path.to_str().unwrap();

                    if let Some(ref mut vols) = apache_service.volumes {
                        vols.push(format!("{}:/opt/docker/etc/httpd/ssl/server.crt:ro", crt_path_usable));
                        vols.push(format!("{}:/opt/docker/etc/httpd/ssl/server.key:ro",  key_path_usable));
                    }

                    if let Some(ref mut ports) = apache_service.ports {
                        ports.push(format!("443:443"));
                    }   
                }  else if self.cluster_config.ssl_certificate_path_crt.is_some() && self.cluster_config.ssl_certificate_path_key.is_none() {
                    println!("Seul le certificat TLS a été renseigné il manque la clé (--ssl_certificate_path_key) !");
                } else if self.cluster_config.ssl_certificate_path_key.is_some() && self.cluster_config.ssl_certificate_path_crt.is_none() {
                    println!("Seul la clé TLS a été renseignée il manque le certificat (--ssl_certificate_path_crt)!");
                }

                if self.cluster_config.services.traefik {
                    apache_service.labels = Some(vec![
                        "traefik.enable=true".to_string(),
                        "traefik.http.routers.apache.rule=Host(`apache.localhost`)".to_string(),
                        "traefik.http.services.apache.loadbalancer.server.port=80".to_string(),
                    ]);
                }

                self.compose
                    .services
                    .insert("apache".to_string(), apache_service);
            }

            Some(ServerType::NodeJS) => {
                let mut node_service = DockerComposeService {
                    image: "node:22".to_string(),
                    labels: None,
                    command: Some(vec!["bash -c 'npm install && npm start'".to_string()]),
                    ports: Some(vec!["3000:3000".to_string()]),
                    volumes: None,
                    environment: None,
                    depends_on: None,
                };

                if self.cluster_config.services.traefik {
                    node_service.labels = Some(vec![
                        "traefik.enable=true".to_string(),
                        "traefik.http.routers.node.rule=Host(`node.localhost`)".to_string(),
                        "traefik.http.services.node.loadbalancer.server.port=80".to_string(),
                    ]);
                }

                self.compose
                    .services
                    .insert("node".to_string(), node_service);
            }

            None => {
                return;
            }
        }
    }

    fn add_database_service(&mut self) {
        let config = envy::from_env::<EnvConfig>().expect("Failed to deserialize config");
        let database_user = config.database_user.unwrap_or_else(|| "appuser".into());
        let database_password = config.database_password.unwrap_or_else(|| "app".into());
        let database_name = config.database_name.unwrap_or_else(|| "app".into());

        match self.cluster_config.services.database {
            Some(DatabaseType::MySQL) => {
                let mut mysql_env = HashMap::new();

                mysql_env.insert(
                    "MYSQL_ROOT_PASSWORD".into(),
                    config
                        .database_rootpassword
                        .unwrap_or_else(|| "rootpassword".into()),
                );
                mysql_env.insert("MYSQL_USER".into(), database_user.clone());
                mysql_env.insert("MYSQL_PASSWORD".into(), database_password.clone());
                mysql_env.insert("MYSQL_DATABASE".into(), database_name.clone());

                let mysql_service = DockerComposeService {
                    image: "mysql:8.4".to_string(),
                    environment: Some(mysql_env),
                    ports: Some(vec!["3306:3306".to_string()]),
                    volumes: Some(vec!["mysql_data:/var/lib/mysql".to_string()]),
                    command: None,
                    labels: None,
                    depends_on: None,
                };

                self.compose
                    .services
                    .insert("mysql".to_string(), mysql_service);

                // Add MySQL exporter
                let mut exporter_env = HashMap::new();
                exporter_env.insert(
                    "DATA_SOURCE_NAME".to_string(),
                    format!("{}:{}@(mysql:3306)/app", database_user, database_password),
                );

                let mut mysqld_exporter = DockerComposeService {
                    image: "prom/mysqld-exporter:latest".to_string(),
                    environment: Some(exporter_env),
                    depends_on: Some(vec!["mysql".to_string()]),
                    labels: None,
                    command: None,
                    ports: None,
                    volumes: None,
                };

                if self.cluster_config.services.traefik {
                    mysqld_exporter.labels = Some(vec![
                        "traefik.enable=true".to_string(),
                        "traefik.http.routers.mysqlmetrics.rule=Host(`mysql-metrics.localhost`)"
                            .to_string(),
                        "traefik.http.services.mysqlmetrics.loadbalancer.server.port=9104"
                            .to_string(),
                    ]);
                }

                self.compose
                    .services
                    .insert("mysqld_exporter".to_string(), mysqld_exporter);

                let mut sql_volume = HashMap::new();
                sql_volume.insert(
                    "mysql_data".to_string(),
                    Value::Mapping(serde_yaml::Mapping::new()),
                );

                let _ = self.compose.volumes.insert(sql_volume);
            }

            Some(DatabaseType::PostgreSQL) => {
                let mut postgres_env = HashMap::new();
                postgres_env.insert("POSTGRES_DB".into(), database_name.clone());
                postgres_env.insert("POSTGRES_USER".to_string(), database_user.clone());
                postgres_env.insert("POSTGRES_PASSWORD".to_string(), database_password.clone());

                let postgres_service = DockerComposeService {
                    image: "postgres:15".to_string(),
                    environment: Some(postgres_env),
                    ports: Some(vec!["5432:5432".to_string()]),
                    volumes: Some(vec!["postgres_data:/var/lib/postgresql/data".to_string()]),
                    command: None,
                    labels: None,
                    depends_on: None,
                };

                self.compose
                    .services
                    .insert("postgres".to_string(), postgres_service);

                let mut postgres_volume = HashMap::new();
                postgres_volume.insert(
                    "postgres_data".to_string(),
                    Value::Mapping(serde_yaml::Mapping::new()),
                );

                let _ = self.compose.volumes.insert(postgres_volume);
            }

            Some(DatabaseType::MongoDB) => {
                let mut mongo_env = HashMap::new();
                mongo_env.insert("MONGO_INITDB_ROOT_USERNAME".to_string(), "root".to_string());
                mongo_env.insert("MONGO_INITDB_ROOT_PASSWORD".to_string(), database_password);

                let mongo_service = DockerComposeService {
                    image: "mongo:7".to_string(),
                    environment: Some(mongo_env),
                    ports: Some(vec!["27017:27017".to_string()]),
                    volumes: Some(vec!["mongo_data:/data/db".to_string()]),
                    command: None,
                    labels: None,
                    depends_on: None,
                };

                self.compose
                    .services
                    .insert("mongo".to_string(), mongo_service);

                self.volumes.insert(
                    "mongo_data".to_string(),
                    Value::Mapping(serde_yaml::Mapping::new()),
                );
            }

            None => {
                return;
            }
        }
    }
}

pub fn generate_docker_file(config: &ClusterConfig) -> io::Result<()> {
    // On met à jour le fichier de config en fonction des services sélectionnées
    let mut docker_compose_builder = DockerComposeBuilder {
        cluster_config: &config,
        volumes: HashMap::new(),
        compose: DockerCompose {
            version: "3.9".to_string(),
            services: HashMap::new(),
            volumes: None,
        },
    };
    let docker_file_content = docker_compose_builder.generate_docker_compose();
    match docker_file_content {
        Ok(docker_file_content) => {
            if let Err(_err) = create_docker_file(&docker_file_content) {
                println!("Erreur lors de la génération du fichier docker file.");
            }
        }
        Err(e) => println!(
            "Erreur lors de la génération du fichier docker compose: {:?}",
            e
        ), // No need to return anything, just handle the error here.
    }
    Ok(())
}

pub fn create_docker_file(dockerfile_content: &str) -> io::Result<()> {
    let env = EnvVariables {};
    match fs::write(&env.get_docker_file_path(), dockerfile_content) {
        Ok(ok) => ok,
        Err(err) => {
            println!("Error while creating : docker-compose.file : {}", err);
            return Err(err.into());
        },
    }
    Ok(())
}
