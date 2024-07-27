import { AouServer } from "../index.js";

const server = new AouServer({
  json: true,
});

server.get("/", async (req) => {
  return {
    status: 200,
    data: {
      path: req.path,
      data: Math.random() * 1000,
    },
  };
});
