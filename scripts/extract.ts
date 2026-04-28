import { execSync } from "child_process";
import { readFile, writeFile, rm } from "fs/promises";
import { resolve } from "path";

const mapName = "sr_seasonal_map";

execSync("pnpm extract");

execSync(
  `gltf-transform optimize assets/maps/${mapName}/mapgeo.glb assets/maps/output.gltf --compress draco --texture-compress ktx2`,
  {
    stdio: "inherit",
  },
);

const data = await readFile("assets/maps/output.gltf", {
  encoding: "utf-8",
});

const json = JSON.parse(data);

json.textures.forEach((texture: any) => {
  if (!texture.extensions) return;

  const extension = texture.extensions.KHR_texture_basisu || texture.extensions.EXT_texture_webp;
  if (!extension) return;

  texture.source = extension.source;
});

await writeFile("assets/maps/output.gltf", JSON.stringify(json));

execSync(`gltf-pipeline -i assets/maps/output.gltf -o assets/maps/${mapName}/mapgeo.glb`, {
  stdio: "inherit",
});

await rm(resolve("assets/maps", "output.gltf"));
await rm(resolve("assets/maps", "output.bin"));
json.images.forEach(async (v: any) => {
  await rm(resolve("assets/maps", v.uri));
});
