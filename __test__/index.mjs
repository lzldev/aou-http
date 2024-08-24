import { AouError, AouServer, AouMiddleware } from "../index.js";
import { readFile } from "node:fs/promises";

const server = new AouServer();

const html = (args) => `
<html>
    <head>
      <title>${args.title}</title>
    </head>
    <body>
      <h1>${args.message}</h1>
    </body>
</html>
`;

const messages = ["Hello World", "How's the Weather?", "yippieeeeeee"];

const image = await readFile("./fixtures/image.png");
server.get("/image", async (req) => {
  return {
    headers: {
      "Content-Type": "image/png",
    },
    body: image,
  };
});

server.get("/", async (req) => {
  return {
    headers: {
      "Set-Cookie": `invalid=12314124\;asdfasdfasdf\ \!!!!`,
    },
    body: html({
      title: Math.random(),
      message: messages[Math.floor(Math.random() * messages.length)],
    }),
  };
});

server.get("/server_error", async (req) => {
  throw new Error("Custom Server Error");
});

server.get("/error", async (req) => {
  throw new AouError({
    headers: {
      "Content-type": "text/html",
    },
    body: html({
      title: Math.random().toFixed(2),
      message: messages[Math.floor(Math.random() * messages.length)],
    }),
  });
});

server.get("/json", async (req) => {
  return {
    body: {
      classic: true,
    },
  };
});

server.get("/{*file}", async (req) => {
  return {
    status: 200,
    body: {
      params: req.params,
      path: req.path,
      file: req.params.file,
      data: Math.random() * 1000,
    },
  };
});

server.all("/not-found", async (req, context) => {
  return {
    status: 404,
    body: "ooops",
  };
});

const chain = AouMiddleware.create(async (req, context) => {
  return {
    req,
    context: { userId: 1 },
  };
}).with(async (req, context) => {
  return {
    req,
    context: { ...context, name: "123" },
  };
});

server.get(
  "/hello/{12345}",
  chain.handle(async (req, context) => {
    return {
      body: "",
    };
  })
);

const instance = await server.listen("0.0.0.0", 7070);
console.log(`Listening on ${instance.ip}:${instance.port}`);
