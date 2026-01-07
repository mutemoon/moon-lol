# moon-lol

## 项目简介 / Project Introduction

`moon-lol` 是一个基于 Bevy 引擎开发的《英雄联盟》复刻项目。该项目利用 Rust 语言的高性能和 Bevy 引擎的现代 ECS 架构，实现了对 LoL 资源文件（WAD, PROP, TEX 等）的解析加载，并初步构建了战斗系统、技能系统以及 UI 界面。

`moon-lol` is a League of Legends remake project developed using the Bevy engine. Leveraging the high performance of Rust and Bevy's modern ECS architecture, this project implements the parsing and loading of LoL resource files (WAD, PROP, TEX, etc.) and has initially built the combat system, skill system, and UI interface.

## 快速开始 / Quick Start

请按照以下步骤准备资源并运行游戏示例：

Please follow these steps to prepare resources and run the game examples:

### 1. 提取资源文件 / Extract Resource Files

在运行游戏之前，需要从本地的《英雄联盟》客户端中提取资源文件。

Before running the game, you need to extract resource files from your local League of Legends client.

```bash
cargo run --example extract
```

> **注意**：运行前请确保 `examples/extract.rs` 中的 `root_dir` 路径指向你本地的英雄联盟游戏目录（例如 `D:\WeGameApps\英雄联盟\Game`）。提取后的资源将存放在 `assets/data` 目录下。
>
> **Note**: Before running, ensure that the `root_dir` path in `examples/extract.rs` points to your local League of Legends game directory (e.g., `D:\WeGameApps\League of Legends\Game`). The extracted resources will be stored in the `assets/data` directory.

### 2. 开始游戏 / Start Game

资源提取完成后，可以运行锐雯（Riven）的技能测试示例来开始游戏：

Once the resource extraction is complete, you can run the Riven skill test example to start the game:

```bash
cargo run --example riven --release
```

该示例将加载锐雯的模型、动画及技能逻辑，你可以在窗口中进行操作测试。

This example will load Riven's model, animations, and skill logic, allowing you to perform operational tests in the window.

## 致谢与参考 / Acknowledgements and References

本项目在开发过程中参考或使用了以下开源项目，感谢这些社区贡献者的工作：

This project referenced or used the following open-source projects during development. We thank these community contributors for their work:

- **CDragon**: 提供了详尽的资源数据参考。 / Provides detailed resource data references.
- **LeagueToolkit**: 资源解析与处理工具。 / Resource parsing and processing tools.
- **LoL-NGRID-converter**: 导航网格转换相关技术实现。 / Technical implementation related to navigation grid conversion.

## 版权声明 / Copyright Notice

**请务必阅读以下声明： / Please be sure to read the following statement:**

1. 本项目仅供技术研究与学习使用，仓库内**不包含**任何《英雄联盟》（League of Legends）的原生美术资源。
   This project is for technical research and learning purposes only. The repository **does not contain** any native art assets from League of Legends.
2. 一切相关美术资源及配置结构体代码的版权均归 **Riot Games（拳头游戏）** 所有。
   The copyright for all related art assets and configuration structure code belongs to **Riot Games**.
3. Riot Games 拥有随时要求关闭或停止此项目的权利。
   Riot Games reserves the right to request the closure or termination of this project at any time.
