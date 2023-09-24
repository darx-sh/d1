export function Hi(msg) {
  log.debug("this is a debug log");
  log.info("this is another log");
  return "Hi " + msg + " from foo, env key1 = " + Darx.env.key1;
}

export function ThrowExp() {
  throw Error('Something went wrong');
}