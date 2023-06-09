// foo.js
function bar() {
  return "/foo.bar";
}
function foo() {
  return "/foo.foo";
}
function fooWithParam(param) {
  return param;
}
function foo_default(param) {
  return "/foo";
}
export {
  bar,
  foo_default as default,
  foo,
  fooWithParam
};
