/// <reference lib="deno.ns" />

await Deno.mkdir("./gen/types", { recursive: true });
// await Deno.mkdir('./src/validators', { recursive: true });

const GenTypesPath = await Deno.realPath("./gen/types");
// const GenJsonSchemasPath = await Deno.realPath('./gen/schemas');
// const GenZodSchemasPath = await Deno.realPath('./src/validators');

const libPath = await Deno.realPath("./src/lib.ts");

console.log("[Task] Removing old bindings...");
await Deno.remove(GenTypesPath, { recursive: true });
// await Deno.remove(GenZodSchemasPath, { recursive: true });
// recreate
await Deno.mkdir(GenTypesPath, { recursive: true });
// await Deno.mkdir(GenZodSchemasPath, { recursive: true });

{
  console.log("[Task] Generating Typescript Bindings and JSON Schemas...");
  // yeah cargo test generates the typescript bindings, why? ask to @aleph-alpha/ts-rs xd
  // btw internally we also decided to use tests to avoid having a binary.
  // also this gill generate the json schemas
  await new Deno.Command("cargo", {
    args: ["test", "--no-default-features"],
    stderr: "inherit",
    stdout: "inherit",
  }).output();
}

/* {
  console.log('[Task] Converting JSON Schemas to Zod Schemas...');
  for (const file of Deno.readDirSync(GenJsonSchemasPath)) {
    if (file.isFile && file.name.endsWith('.schema.json')) {
      const schema = JSON.parse(await Deno.readTextFile(`${GenJsonSchemasPath}/${file.name}`));
      const { resolved } = await resolveRefs(schema);

      const zodCode = jsonSchemaToZod(resolved, { module: 'esm' });
      await Deno.writeTextFile(`${GenZodSchemasPath}/${file.name.replace('.schema.json', '.ts')}`, zodCode);
    }
  }
} */

{
  console.log("[Task] Creating entry points...");
  /* const zodMod = await Deno.open(`${GenZodSchemasPath}/mod.ts`, {
    create: true,
    append: true,
  });
  for (const file of Deno.readDirSync(GenZodSchemasPath)) {
    if (file.isFile && file.name.endsWith('.ts') && file.name !== 'mod.ts') {
      await zodMod.write(
        new TextEncoder().encode(`export { default as ${file.name.replace('.ts', '')} } from './${file.name}';\n`),
      );
    }
  } */

  const typesMod = await Deno.open(`${GenTypesPath}/mod.ts`, {
    create: true,
    append: true,
  });
  for (const entry of Deno.readDirSync(GenTypesPath)) {
    if (entry.isFile && entry.name.endsWith(".ts") && entry.name !== "mod.ts") {
      await typesMod.write(
        new TextEncoder().encode(`export * from './${entry.name}';\n`),
      );
    }
  }
}

{
  console.log("[Task] Extracting Types Definitions...");
  const doc = await new Deno.Command("deno", {
    args: ["doc", "--json", "--private", `${GenTypesPath}/mod.ts`],
    stderr: "inherit",
    stdout: "piped",
  }).output();
  const docJson = JSON.parse(new TextDecoder().decode(doc.stdout));
  await Deno.writeTextFile(
    "./gen/doc-types.json",
    JSON.stringify(docJson, null, 2),
  );
}

{
  console.log("[Task] Extracting Library Definitions...");
  const doc2 = await new Deno.Command("deno", {
    args: ["doc", "--json", "--private", libPath],
    stderr: "inherit",
    stdout: "piped",
  }).output();
  const docJson2 = JSON.parse(new TextDecoder().decode(doc2.stdout));
  await Deno.writeTextFile(
    "./gen/doc-lib.json",
    JSON.stringify(docJson2, null, 2),
  );
}

console.log("[Task] Formatting...");
await new Deno.Command("cargo", {
  args: ["fmt"],
  stderr: "inherit",
  stdout: "inherit",
}).output();
await new Deno.Command("deno", {
  args: ["fmt", "--quiet"],
  stderr: "inherit",
  stdout: "inherit",
}).output();
console.log("[Task] Done!");
