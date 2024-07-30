# aou-http

`aou-http` is a Node.js http server written in Rust, made possible by FFI bindings

## Getting Started

```bash
npm install @aou-http/server
# or
yarn add @aou-http/server
# or
pnpm add @aou-http/server
```

### Minimal Example

```javascript
import { AouServer } from "@aou-http/server";

const server = new AouServer();

server.get("/", async (req) => {
  return {
    headers: {
      "Content-type": "application/json",
    },
    body: {
      lucky_number: Math.random(),
      message: `Hello ${req.query.user || "Stranger"}`,
    },
  };
});

const { ip, port } = await server.listen("0.0.0.0", 7070);

console.info(`Server Running on ${ip}:${port}`);
```
