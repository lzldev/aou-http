import process from "node:process";
import test, { registerCompletionHandler } from "ava";

import { AouRequest, AouServer } from "../index.js";

const [addr, port] = ["0.0.0.0", 7070];

let server;
test.before("setup server", async () => {
  server = new AouServer();

  server.get("/route/{file}", async (req) => {
    const param = req.params.file;
    const query = req.query.query;
    const header = req.headers["x-test-header"];

    return {
      body: {
        param,
        query,
        header,
      },
    };
  });

  const instance = await server.listen(addr, port);
});

test("req.params , req.query , req.header", async (t) => {
  const test_param = "f";
  const test_query = "test-query";
  const test_header = "test-header-body";

  const res = await fetch(
    `http://${addr}:${port}/route/${test_param}?test=12345?&query=${test_query}`,
    {
      headers: {
        "x-test-header": test_header,
      },
    }
  );

  const body = await res.json();

  t.is(res.status, 200);

  t.truthy(body);
  t.is(body.param, test_param);
  t.is(body.query, test_query);
  t.is(body.header, test_header);
});

test("404", async (t) => {
  const not_found_res = await fetch(`http://${addr}:${port}/invalid-route`);

  t.is(not_found_res.status, 404);
  t.is(not_found_res.statusText, "Not Found");
});

test("request parsing", async (t) => {
  const request = AouRequest.fromString(
    `GET / HTTP/1.1\r\nHost: localhost:7070\r\nContent-Length: 25\r\n\r\n`
  );

  t.is(request.method, "GET");
  t.is(request.path, "/");
});

registerCompletionHandler(() => process.exit(0));
