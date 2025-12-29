use std::collections::HashMap;
use serde::{Deserialize, Serialize};

struct Service {
    name: String,
    image: String,
    command: Option<Vec<String>>,
    ports: Option<Vec<String>>,
    volumes: Option<Vec<String>>,
    environment: Option<HashMap<String, String>>,
    labels: Option<Vec<String>>,
    requiredServices: Option<Vec<String>>,
    networks: Option<Vec<String>>
}

impl Clone for Service {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            image: self.image.clone(),
            command: self.command.clone(),
            ports: self.ports.clone(),
            volumes: self.volumes.clone(),
            environment: self.environment.clone(),
            labels: self.labels.clone(),
            requiredServices: self.requiredServices.clone(),
            networks: self.networks.clone(),
        }
    }
}

impl Service {
    pub fn toDockerService(self) -> DockerService {
        let service = DockerService {
            image: self.image,
            command: self.command.clone(),
            ports: self.ports.clone(),
            volumes: self.volumes.clone(),
            labels: self.labels.clone(),
            environment: self.environment.clone(),
            depends_on: self.requiredServices.clone(),
            networks: self.networks.clone(),
        };
        service
    }
}

#[derive(Serialize, Deserialize)]
struct DockerService {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    networks: Option<Vec<String>>
}

struct CLIServiceArgs {
    server: Option<String>,
    database: Option<String>,
    traefik: bool,
    dashboard: bool,
}

struct ServiceManager {}
impl ServiceManager {
    pub fn buildServices(self, args: CLIServiceArgs) -> Vec<Service> {
        let mut services:Vec<Service> = vec![];
        if args.traefik {
            services = [&ServiceManager::buildTraefikServices()[..], &services[..]].concat();
        }

        if args.database.is_some() {
            services = [&ServiceManager::buildDatabaseServices(args.database.expect("Databases services not defined"))[..], &services[..]].concat();
        }

        if args.server.is_some() {
            services = [&ServiceManager::buildServersServices(args.server.expect("Servers services were not defined"))[..], &services[..]].concat();
        }

        if args.dashboard {
            services = [&ServiceManager::buldDashboardServices()[..], &services[..]].concat();
        }
        return vec![];
    }

    pub fn buldDashboardServices() -> Vec<Service> {
        return vec![];
    }

    pub fn buildDatabaseServices(databases: String) -> Vec<Service> {
        return vec![];
    }

    pub fn buildServersServices(servers: String) -> Vec<Service> {
        return vec![];
    }

    pub fn buildTraefikServices() -> Vec<Service> {
        return vec![];
    }
}

