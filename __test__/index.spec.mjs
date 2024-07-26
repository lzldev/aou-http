import test from "ava";

import { AouRequest, AouServer } from "../index.js";

test("initialize server", async (test) => {
  const server = new AouServer();

  test.truthy(server);
  test.false(server.isRunning());

  let counter = 0;
  server.get("/", (...funny) => {
    console.info("Args,", funny);
    console.log("Hello World from server ", counter);
    counter++;
  });

  // await server.fakeListen();

  // server.listen("127.0.0.1", 8080);
});

test("requets parsing", async (test) => {
  const request = AouRequest.fromString(
    `GET / HTTP/1.1\r\nHost: localhost:7070\r\nContent-Length: 25\r\n\r\n`
  );

  test.is(request.method, "GET");
  test.is(request.path, "/");
});
