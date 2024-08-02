//FROM -- ./extend.js

module.exports.AouError = class AouError extends Error {
  name = "AouError";

  constructor(value) {
    super();
    this.message = JSON.stringify(value);
  }
};

module.exports.AouMiddleware = class AouMiddleware {
  handlers;

  constructor(handlers) {
    this.handlers = handlers;
  }

  static create(handler) {
    return new AouMiddleware([handler]);
  }

  with(handler) {
    return new AouMiddleware([...this.handlers, handler]);
  }

  handle(handler) {
    return async (req) => {
      let r = {
        req,
        context: {},
      };

      for (const middleware of this.handlers) {
        r = await middleware(r.req, r.context);
      }

      return handler(r.req, r.context);
    };
  }
};
