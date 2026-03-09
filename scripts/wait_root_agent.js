const net = require("net");

// Keep these values in sync with backend/src/services/root_agent.rs
const SOCKET_PATH = "/run/mana-panel/root-agent.sock";
const SHARED_TOKEN = "mana-panel-root-agent-fixed-token";
const READY_TIMEOUT_MS = Number.parseInt(
    process.env.WAIT_ROOT_AGENT_TIMEOUT_MS || "60000",
    10,
);
const RETRY_INTERVAL_MS = 500;
const CONNECT_TIMEOUT_MS = 1_500;

function waitForRootAgent() {
    return new Promise((resolve, reject) => {
        const deadline = Date.now() + READY_TIMEOUT_MS;

        const attempt = () => {
            const client = net.createConnection({ path: SOCKET_PATH });
            let finished = false;
            let responseBuffer = "";

            const finish = (err) => {
                if (finished) {
                    return;
                }

                finished = true;
                client.destroy();

                if (!err) {
                    resolve();
                    return;
                }

                if (Date.now() >= deadline) {
                    reject(new Error(`root-agent not ready: ${err.message}`));
                    return;
                }

                setTimeout(attempt, RETRY_INTERVAL_MS);
            };

            client.setEncoding("utf8");
            client.setTimeout(CONNECT_TIMEOUT_MS);

            client.on("connect", () => {
                const payload = JSON.stringify({
                    token: SHARED_TOKEN,
                    command: { kind: "ping" },
                });
                client.write(`${payload}\n`);
            });

            client.on("data", (chunk) => {
                responseBuffer += chunk;
                const newline = responseBuffer.indexOf("\n");
                if (newline === -1) {
                    return;
                }

                const line = responseBuffer.slice(0, newline).trim();
                try {
                    const response = JSON.parse(line);
                    if (response && response.success === true) {
                        finish();
                    } else {
                        finish(
                            new Error(
                                response?.message ||
                                    "root-agent returned failure",
                            ),
                        );
                    }
                } catch (err) {
                    finish(err);
                }
            });

            client.on("timeout", () => finish(new Error("connect timeout")));
            client.on("error", (err) => finish(err));
            client.on("end", () =>
                finish(new Error("connection closed before response")),
            );
        };

        attempt();
    });
}

console.log("waiting root-agent...");
waitForRootAgent()
    .then(() => {
        console.log("root-agent ready");
    })
    .catch((err) => {
        console.error(err.message);
        process.exit(1);
    });
