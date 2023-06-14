import hello from "../lib/hello.js";

export function bar() {
  let h = hello();
  return "/foo.bar";
}

export function foo() {
  return "/foo.foo";
}

export function fooWithParam(param) {
  return param;
}

export default function (param) {
  return "/foo";
}
