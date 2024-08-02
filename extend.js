//FROM -- ./extend.js

module.exports.AouError = class AouError extends Error {
  name = "AouError";

  constructor(value) {
    super();
    this.message = JSON.stringify(value);
  }
};
