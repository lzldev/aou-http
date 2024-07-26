import test from "ava";

import { AouRequest, AouServer } from "../index.js";

test("initialize server", async (test) => {
  const server = new AouServer();

  test.truthy(server);

  let counter = 0;

  server.get("/", (req) => {
    return {
      Hello: "Hello",
      number: Math.random() * 20,
      inner: {
        data: 1234,
      },
    };
  });

  const instance = await server.listen("0.0.0.0", 7070);
});

test("requets parsing", async (test) => {
  const request = AouRequest.fromString(
    `GET / HTTP/1.1\r\nHost: localhost:7070\r\nContent-Length: 25\r\n\r\n`
  );

  test.is(request.method, "GET");
  test.is(request.path, "/");
});
