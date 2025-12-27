import express from "express";
import cors from "cors";
console.log("DOCKER_AGENT_URL : ", process.env.AGENT_URL);
console.log("FRONTEND_URL : ", process.env.FRONTEND_URL);
const app = express();
app.use(cors({
    origin: process.env.FRONTEND_URL,
}));
app.get("/api/docker/health", async (_req, res) => {
    try {
        const url = process.env.AGENT_URL + "/api/docker/health";
        const health_data_raw = await fetch(url);
        console.log("health_data_raw : ", health_data_raw);
        const health_data = await health_data_raw.json();
        console.log("health_data : ", health_data);
        res.status(200).json({
            connected: true,
            swarmActive: health_data.Swarm?.LocalNodeState === 'active',
            swarmNodeId: health_data.Swarm?.NodeID || null,
            swarmManagers: health_data.Swarm?.Managers || 0,
            swarmNodes: health_data.Swarm?.Nodes || 0
        });
    }
    catch (error) {
        console.error("Docker connection error:", error);
        const errorMessage = error instanceof Error ? error.message : String(error);
        res.status(500).json({
            connected: false,
            error: errorMessage
        });
    }
});
// Fetch all nodes
app.get("/api/docker/nodes", async (_req, res) => {
    try {
        console.log(process.env.AGENT_URL + "/api/docker/nodes");
        const nodes_data_raw = await fetch(process.env.AGENT_URL + "/api/docker/nodes");
        console.log("nodes_data_raw : ", nodes_data_raw);
        const nodes_data = await nodes_data_raw.json();
        console.log("nodes_data", nodes_data);
        const nodes_formatted = nodes_data.map((n) => ({
            id: n.ID,
            hostname: n.Description?.Hostname || "unknown",
            status: n.Status?.State || "unknown",
        }));
        console.log("nodes_data_formatted : ", nodes_formatted);
        res.status(200).json(nodes_formatted);
    }
    catch (error) {
        console.error("Error fetching nodes:", error);
        res.status(500).json({ error: "Failed to fetch nodes" });
    }
});
// Fetch all services
app.get("/api/docker/services", async (_req, res) => {
    try {
        console.log(process.env.AGENT_URL + "/api/docker/services");
        const services_data_raw = await fetch(process.env.AGENT_URL + "/api/docker/services");
        console.log("services_data_raw : ", services_data_raw);
        const services = await services_data_raw.json();
        console.log("services : ", services);
        const formatted_services = services.map((s) => ({
            id: s.ID,
            name: s.Spec?.Name || "unknown",
            replicas: s.Spec?.Mode?.Replicated?.Replicas?.toString() ??
                "Global / Unknown",
        }));
        console.log("formatted_services : ", formatted_services);
        res.json(formatted_services);
    }
    catch (error) {
        console.error("Error fetching services:", error);
        res.status(500).json({ error: "Failed to fetch services" });
    }
});
// Health check
app.get("/api/health", (_req, res) => {
    res.json({ status: "ok" });
});
const PORT = process.env.PORT || 3001;
app.listen(PORT, () => {
    console.log(`✅ Docker Swarm API server running on http://localhost:${PORT}`);
});
