use crate::ClusterConfig;
use crate::Service;
use crate::utils::command;
use regex::Regex;
use std::fs;
use std::process::Command;
use std::time::Duration;

pub fn check_existing_cluster() -> bool {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("ca")
        .output()
        .unwrap();

    let output_str = String::from_utf8_lossy(&output.stderr);
    if output_str.contains("Error response from daemon") {
        return false;
    }

    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }

    return true;
}

impl ClusterConfig {
    pub fn init(&mut self) -> () {
        let output = Command::new("docker")
            .arg("swarm")
            .arg("init")
            .output()
            .unwrap();

        let output_string = String::from_utf8_lossy(&output.stdout).to_string(); // owns the String

        let output_sentences: Vec<&str> = output_string.lines().collect();

        for sentence in output_sentences {
            if sentence.trim().starts_with("docker swarm join") {
                self.cluster_docker_command = sentence.trim().to_string();
            }
        }

        if !output.status.success() {
            eprintln!("Error:");
            eprintln!("{}", String::from_utf8_lossy(&output.stderr));
        }
    }

    pub fn install_docker(&mut self) {
        for node_config in &self.nodes_configs {
            let target = format!("{}@{}", node_config.username, node_config.ip);

            let mut cmd = Command::new("sshpass");
            cmd.arg("-p")
            .arg(&node_config.password)
            .arg("ssh")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg(&target)
            .arg(format!(
                "echo {} | sudo -S apt-get update -y && echo {} | sudo -S apt-get install -y docker.io",
                node_config.password,
                node_config.password
            ));

            match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
                Ok(Some(output)) => {
                    if output.status.success() {
                        println!("Docker installed on {}", target);
                    } else {
                        println!(
                            "Install failed on {}: {}",
                            target,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for {}", target),
                Err(e) => println!("Execution failed on {}: {}", target, e),
            }
        }
    }

    pub fn join_cluster(&self) {
        for node_config in &self.nodes_configs {
            let target = format!("{}@{}", node_config.username, node_config.ip);
            let command = &self.cluster_docker_command;

            let mut cmd = Command::new("ssh");
            cmd.arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg(&target)
                .arg(format!("{}", command));

            match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
                Ok(Some(output)) => {
                    if output.status.success() {
                        println!("{} joined the cluster", target);
                    } else {
                        println!(
                            "{} failed to join the cluster : {}",
                            target,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for execution of {} on : {}", command, target),
                Err(e) => println!("Execution of {} failed on {} : {}", command, target, e),
            }
        }
    }

    pub fn leave_cluster(&mut self) {
        for node_config in &self.nodes_configs {
            let target = format!("{}@{}", node_config.username, node_config.ip);
            let command = "docker swarm leave --force";

            let mut cmd = Command::new("ssh");
            cmd.arg("-o")
                .arg("StrictHostKeyChecking=no")
                .arg(&target)
                .arg(format!("{}", command));

            match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
                Ok(Some(output)) => {
                    if output.status.success() {
                        println!("{} left the cluster", target);
                    } else {
                        println!(
                            "{} failed to leave the cluster : {}",
                            target,
                            String::from_utf8_lossy(&output.stderr)
                        );
                    }
                }
                Ok(None) => println!("Timeout for execution of {} on : {}", command, target),
                Err(e) => println!("Execution of {} failed on {} : {}", command, target, e),
            }
        }
    }

    pub fn update_docker_config_file(&self) -> std::io::Result<()> {
        let mut template_config_content_result =
            fs::read_to_string("src/docker/docker-compose.example.yml");
        let mut template_config_content = String::from("");

        if template_config_content_result.is_ok() {
            template_config_content = template_config_content_result.unwrap();
        } else {
            panic!("{}", template_config_content_result.unwrap());
        }

        // Mise à jour du nombre de nodes dans le cluster.
        let replicas_text_regexp = Regex::new(r"replicas: \d").unwrap();
        let replicas_text = replicas_text_regexp
            .captures(&template_config_content)
            .ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::Other, "Failed to capture replicas text")
            })?;

        let matched_replicas = replicas_text.get(0).unwrap().as_str();
        println!("mateched replicas : {}", matched_replicas);

        template_config_content = template_config_content.replace(
            matched_replicas,
            &format!("replicas: {}", self.nodes_number),
        );

        // On supprime toutes les parties concernant le service traefik
        if !self.services.contains(&Service::Traefik) {
            let mut result = String::new();
            let mut in_traefik_service_definition = false;

            // On garde toutes les lignes qui ne sont pas comprises entre traefik: (inclus) et
            // nginx: (exclus) dans le docker-compose.yml
            for line in template_config_content.lines() {
                // Si on est dans la section traefik alors on ne conserve pas les lignes dans
                // result.
                if line.contains("traefik:") {
                    in_traefik_service_definition = true;
                } else if in_traefik_service_definition {
                    if line.contains("nginx:") {
                        // Si on est plus dans le section traefik alors on conserve les
                        // lignes.
                        in_traefik_service_definition = false;
                        result.push_str(line);
                        result.push('\n');
                    }
                } else {
                    // Si on est pas dans la section traefik alors on conserve les lignes.
                    result.push_str(line);
                    result.push('\n');
                }
            }
            template_config_content = result;

            let mut result = String::new();
            let mut in_labels_definition = false;
            for line in template_config_content.lines() {
                if line.contains("labels:") {
                    in_labels_definition = true;
                    continue;
                }
                if in_labels_definition {
                    if !line.trim().starts_with("- ") {
                        in_labels_definition = false;
                        result.push_str(line);
                        result.push_str("\n");
                    }

                    // Si on est toujours dans la section des labels alors on skip la ligne
                    continue;
                }
                result.push_str(line);
                result.push('\n');
            }

            template_config_content = result;
        }

        // On met à jour le fichier docker compose avec les nouvelles données.
        println!("new template_config_content : {}", template_config_content);
        fs::write("src/docker/docker-compose.yml", &template_config_content);
        Ok(())
    }
}

pub fn destroy_cluster() -> () {
    let output = Command::new("docker")
        .arg("swarm")
        .arg("leave")
        .arg("--force")
        .output()
        .unwrap();

    if !output.status.success() {
        eprintln!("Error:");
        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

pub fn deploy_services() {
    let mut cmd = Command::new("docker");
    cmd.arg("stack")
        .arg("deploy")
        .arg("-c")
        .arg("config.yaml")
        .arg("server");

    match command::run_with_timeout(cmd, Duration::from_secs(1000)) {
        Ok(Some(output)) => {
            if output.status.success() {
                println!("Services deployed.");
            } else {
                println!(
                    "Failed to deploy services : {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Ok(None) => {
            println!("Timeout for execution of 'docker stack deploy -c config.yaml server' ")
        }
        Err(e) => println!(
            "Execution of 'docker stack deploy -c config.yaml server' failed: {}",
            e
        ),
    }
}
