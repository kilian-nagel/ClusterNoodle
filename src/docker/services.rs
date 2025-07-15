use crate::config::config;
use std::fs::File;
use std::io::Write;

pub fn create_docker_config_file(config: &config::ClusterConfig) -> std::io::Result<()> {
    let nodes_number: u16 = config.nodes_number;
    let mut file = File::create("config.yaml")?;
    let file_content = format!(
        "
services:
  web:
    image: nginx
    deploy:
      replicas: {}
      update_config:
        parallelism: 2
        delay: 10s
    ports:
      - \"80:80\"
  db:
    image: postgres
    environment:
      POSTGRES_PASSWORD: admin

networks:
  net: 

volumes:
  db_data:
",
        nodes_number,
    );
    file.write_all(&file_content.into_bytes())?;
    Ok(())
}
