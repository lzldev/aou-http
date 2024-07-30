import { AouServer } from "../index.js";

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

server.get("/", async (req) => {
  //prettier-ignore
  return {
    headers:{
      "Content-type":"text/html"
    },
    body:html({
      title:Math.random().toFixed(2),
      message:messages[Math.floor(Math.random() * messages.length)]
    })
  }
});

server.get("/", async (req) => {
  return {
    headers: {
      "Content-type": "application/json",
    },
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

await server.listen("0.0.0.0", 7070);
