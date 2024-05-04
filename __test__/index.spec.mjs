import test from "ava";

import { sum } from "../index.js";

test("sum from native", (t) => {
  t.is(sum(1, 2), 3);
});

import { AouServer } from "../index.js";

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

  await server.fakeListen();

  // server.listen("127.0.0.1", 8080);
});
