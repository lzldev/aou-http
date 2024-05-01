import test from "ava";

import { sum } from "../index.js";

test("sum from native", (t) => {
  t.is(sum(1, 2), 3);
});

import { AouServer } from "../index.js";

test("initialize server", (t) => {
  const server = new AouServer();

  t.truthy(server);

  console.log(server.isRunning());

  console.log("hello world");
  console.log("hello world");
  console.log("hello world");
  console.log("hello world");
  t.false(server.isRunning());

  server.get("/", () => {
    console.log("Hello World from server");
  });

  server.fakeListen();
});
