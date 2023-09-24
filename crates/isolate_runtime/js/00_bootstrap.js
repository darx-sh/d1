import * as console from "ext:deno_console/02_console.js";
import * as fetch from "ext:deno_fetch/26_fetch.js";
import * as headers from "ext:deno_fetch/20_headers.js";
import * as formData from "ext:deno_fetch/21_formdata.js";
import * as response from "ext:deno_fetch/23_response.js";
import * as request from "ext:deno_fetch/23_request.js";

const core = Deno.core;
const ops = core.ops;

const {
  ArrayPrototypeIndexOf,
  ArrayPrototypePush,
  ArrayPrototypeShift,
  ArrayPrototypeSplice,
  Error,
  ErrorPrototype,
  ObjectDefineProperty,
  ObjectDefineProperties,
  ObjectPrototypeIsPrototypeOf,
  ObjectSetPrototypeOf,
  ObjectFreeze,
  SafeWeakMap,
  StringPrototypeSplit,
  WeakMapPrototypeGet,
  WeakMapPrototypeSet,
  WeakMapPrototypeDelete,
} = globalThis.__bootstrap.primordials;

function nonEnumerable(value) {
  return {
    value,
    writable: true,
    enumerable: false,
    configurable: true,
  };
}

function writable(value) {
  return {
    value,
    writable: true,
    enumerable: true,
    configurable: true,
  };
}

const globalScope = {
  console: nonEnumerable(
    new console.Console((msg, level) => core.print(msg, level > 1))
  ),
  // fetch
  Request: nonEnumerable(request.Request),
  Response: nonEnumerable(response.Response),
  Headers: nonEnumerable(headers.Headers),
  fetch: writable(fetch.fetch),
};

ObjectDefineProperties(globalThis, globalScope);
globalThis.Darx = {};