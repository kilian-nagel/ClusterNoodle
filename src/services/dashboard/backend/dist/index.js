import express from "express";
import Docker from "dockerode";
import cors from "cors";
const app = express();
app.use(cors());
// Connect to Docker socket
const docker = new Docker({ socketPath: "/var/run/docker.sock" });
app.get("/api/docker/health", async (_req, res) => {
    try {
        const info = await docker.info();
        res.json({
            connected: true,
            swarmActive: info.Swarm?.LocalNodeState === 'active',
            swarmNodeId: info.Swarm?.NodeID || null,
            swarmManagers: info.Swarm?.Managers || 0,
            swarmNodes: info.Swarm?.Nodes || 0
        });
    }
    catch (error) {
        console.error("Docker connection error:", error);
        res.status(500).json({
            connected: false,
            error: error.message
        });
    }
});
// Fetch all nodes
app.get("/api/docker/nodes", async (_req, res) => {
    try {
        const nodes = await docker.listNodes();
        const formatted = nodes.map((n) => ({
            id: n.ID,
            hostname: n.Description?.Hostname || "unknown",
            status: n.Status?.State || "unknown",
        }));
        res.json(formatted);
    }
    catch (error) {
        console.error("Error fetching nodes:", error);
        res.status(500).json({ error: "Failed to fetch nodes" });
    }
});
// Fetch all services
app.get("/api/docker/services", async (_req, res) => {
    try {
        const services = await docker.listServices();
        const formatted = services.map((s) => ({
            id: s.ID,
            name: s.Spec?.Name || "unknown",
            replicas: s.Spec?.Mode?.Replicated?.Replicas?.toString() ??
                "Global / Unknown",
        }));
        res.json(formatted);
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
