# aou-http

`aou-http` is a Node.js http server written in Rust, made possible by FFI bindings

Currently only available on the following platforms:

- Windows
- Linux Musl
- Linux GNU
- MacOS ARM64
- MacOS x64

## Getting Started

```bash
npm install @aou-http/server
# or
yarn add @aou-http/server
# or
pnpm add @aou-http/server
```

## Minimal Example

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

## Routing

Dynamic routes can be defined by using `{}` inside of the route string.

- `/resource/{id}`
- `/resource/{name}/{id}`

Catch all Routes:

- `resource/{*id}`

`Request.params` will be infered from the route string.

Example:\
Route String: `/resource/{id}/sub-resource/{subId}`
Will generate a Params record of type:

```typescript
{
  id: string;
  subId: string;
}
```

### Methods:

A Catch all route method can be added using the: `server.all()` method.
Methods with more specificity will take precedence over routes with less specificity.

## Throwing HTTP Errors

To throw errors directed towards the client, use the `AouError` class.
Every other error is considered a server error and will be handled by the server.

```javascript
server.get("/", async (req) => {
  throw new AouError({
    status: 404,
    body: {
      message: "Not Found",
    },
  });
});
```

### Middleware

```javascript
// First start building middleware functions
const firstMiddleware = new AouMiddleware(async (req, context) => {
  return {
    req,
    context: { ...context, name: "123" },
  };
});
//those can be chained together
const secondMiddleware = firstMiddleware.with(async (req, context) => {
  return {
    req,
    context: { ...context, id: 1 },
  };
});

//When you are ready to create a handler do

server.get(
  "/route",
  secondMiddleware.handle(async (req, context) => {
    /*
    The type of context here will be 
    {
      id:number,
      name:string
    }
  */
    return {
      body: context,
    };
  })
);
```

- Middlewares can also throw errors at any point in the chain
