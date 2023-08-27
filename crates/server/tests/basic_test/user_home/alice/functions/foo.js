export function Hi(msg) {
  return "Hi " + msg + " from foo, env key1 = " + Darx.env.key1;
}

export function ThrowExp() {
  throw Error('Something went wrong');
}