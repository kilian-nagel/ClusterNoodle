pub struct NodeConfig {
    pub ip: String,
    pub username: String,
    pub password: String,
}

pub struct ClusterConfig {
    pub nodes_number: u16,
    pub nodes_configs: Vec<NodeConfig>,
}
