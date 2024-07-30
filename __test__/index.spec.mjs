import process from "node:process";
import test, { registerCompletionHandler } from "ava";

import { AouRequest, AouServer } from "../index.js";

test("setup server and send request", async (test) => {
  let server = new AouServer();

  test.truthy(server);

  server.get("/route/{file}", async (req) => {
    const file = req.params.file;

    return {
      body: {
        file,
      },
    };
  });

  const [addr, port] = ["0.0.0.0", 7070];
  const instance = await server.listen(addr, port);

  const res = await fetch(`http://${addr}:${port}/route/f`);
  const body = await res.json();

  test.is(res.status, 200);
  test.truthy(body);
  test.assert(body.file === "f");
});

test("requests parsing", async (test) => {
  const request = AouRequest.fromString(
    `GET / HTTP/1.1\r\nHost: localhost:7070\r\nContent-Length: 25\r\n\r\n`
  );

  test.is(request.method, "GET");
  test.is(request.path, "/");
});

registerCompletionHandler(() => process.exit(0));
