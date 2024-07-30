import process from "node:process";
import test, { registerCompletionHandler } from "ava";

import { AouRequest, AouServer } from "../index.js";

let initial_port = 7070;

test("setup server and send request", async (test) => {
  let server = new AouServer();

  test.truthy(server);

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

  const test_param = "f";
  const test_query = "test-query";
  const test_header = "test-header-body";

  const [addr, port] = ["0.0.0.0", initial_port++];
  const instance = await server.listen(addr, port);

  const res = await fetch(
    `http://${addr}:${port}/route/${test_param}?query=${test_query}`,
    {
      headers: {
        "x-test-header": test_header,
      },
    }
  );

  const body = await res.json();

  test.is(res.status, 200);

  test.truthy(body);
  test.is(body.param, test_param);
  test.is(body.query, test_query);
  test.is(body.header, test_header);
});

test("requests parsing", async (test) => {
  const request = AouRequest.fromString(
    `GET / HTTP/1.1\r\nHost: localhost:7070\r\nContent-Length: 25\r\n\r\n`
  );

  test.is(request.method, "GET");
  test.is(request.path, "/");
});

registerCompletionHandler(() => process.exit(0));
