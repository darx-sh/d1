export function handler(param) {
  return 'hi from js foo, this is ' + param.name + ', ' + param.age + " years old";
}
console.log('I am loaded')