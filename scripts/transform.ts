import { Document, NodeIO, Node } from "@gltf-transform/core";
import { ALL_EXTENSIONS } from "@gltf-transform/extensions";
import { readdirSync } from "fs";
import { resolve } from "path";
import * as THREE from "three";

const inputDir = "assets/extract_data";
const outputDir = "assets/transformed_data";

async function transformGLB(filePath: string, filename: string) {
  try {
    // Configure I/O
    const io = new NodeIO().registerExtensions(ALL_EXTENSIONS);

    // Read GLB file
    console.log(`Processing: ${filePath}`);
    const document = await io.read(filePath);

    // Apply transformations using a custom transform function
    await document.transform(applyTransformations());

    // Write to GLB format (single file)
    const outputPath = resolve(outputDir, filename.replace(".glb", ".gltf"));
    const glbBuffer = await io.writeBinary(document);

    // Write the buffer to file
    const fs = await import("fs/promises");
    await fs.writeFile(outputPath, glbBuffer);

    console.log(`已保存文件: ${outputPath}`);
  } catch (error) {
    console.error("An error happened while processing the file:", error);
  }
}

// Custom transform function to apply rotations and scaling
function applyTransformations(): (document: Document) => void {
  return (document: Document): void => {
    const scene = document.getRoot().getDefaultScene();

    if (!scene) {
      console.log("⚠️ No default scene found");
      return;
    }

    // 1. 创建旋转
    // 创建一个空的四元数来累积旋转
    const finalRotation = new THREE.Quaternion();

    // a. 创建绕 X 轴旋转 90 度的四元数
    const rotationX = new THREE.Quaternion();
    rotationX.setFromAxisAngle(new THREE.Vector3(1, 0, 0), -Math.PI / 2);

    // b. 创建绕 Y 轴旋转 180 度的四元数
    const rotationY = new THREE.Quaternion();
    rotationY.setFromAxisAngle(new THREE.Vector3(0, 1, 0), Math.PI);

    // c. 组合旋转：先应用X轴旋转，再应用Y轴旋转
    // 注意：顺序是 final = Y * X
    finalRotation.multiplyQuaternions(rotationY, rotationX);

    // 2. 获取场景的所有根节点
    const rootNodes: Node[] = scene.listChildren();

    // 3. 对每个根节点应用变换
    for (const node of rootNodes) {
      // --- 应用组合后的旋转 ---
      const currentRotation = node.getRotation();
      const currentQuat = new THREE.Quaternion().fromArray(currentRotation);

      // 将我们计算出的最终旋转与节点的当前旋转结合
      const newQuat = new THREE.Quaternion()
        .copy(finalRotation) // 我们新创建的组合旋转
        .multiply(currentQuat); // 乘以节点原有的旋转

      node.setRotation(newQuat.toArray() as [number, number, number, number]);

      // // --- 应用缩放 ---
      const scale = node.getScale();
      // 沿X轴镜像翻转
      scale[0] = -scale[0];
      node.setScale(scale);
    }
  };
}

// 处理assets目录下的所有GLB文件
async function processAllGLBFiles() {
  const files = readdirSync(inputDir);

  console.log(`Found ${files.length} GLB files to process`);

  // Process files sequentially to avoid overwhelming the system
  for (const filename of files) {
    await transformGLB(resolve(inputDir, filename), filename);
  }

  console.log("All files processed successfully!");
}

// 运行程序
processAllGLBFiles().catch(console.error);
