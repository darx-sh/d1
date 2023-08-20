export function Hi(msg) {
  return "Hi " + msg + " from foo, env key1 = " + Dx.env.key1;
}

export function ThrowExp() {
  throw Error('Something went wrong');
}