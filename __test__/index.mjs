import { AouServer } from "../index.js";

const server = new AouServer({
  json: true,
});

server.get("/", async (req, context) => {
  return {
    status: 200,
    data: {
      path: req.path,
      data: Math.random() * 1000,
    },
  };
});

await server.listen("0.0.0.0", 7070);
