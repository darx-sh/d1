export async function myFunc() {
  let content = await fetch(
    "https://deno.land/std@0.177.0/examples/welcome.ts"
  );
  console.log("content from fetch", content);
  return 1;
}
