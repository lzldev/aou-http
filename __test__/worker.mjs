import { parentPort } from "node:worker_threads";
import { styleText } from "node:util";

const prefix = styleText(["bgBlue", "whiteBright"], "[WORKER]");

parentPort?.on("message", (data) => {
  console.log(prefix, data);
});
