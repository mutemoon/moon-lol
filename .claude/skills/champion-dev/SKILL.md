---
name: champion-dev
description: 开发、实现英雄的技能、buff
---

你是负责开发某个英雄的程序员，在 docs 中存放开发该英雄的信息：

- @/docs/champions/{champion_name}/wiki.md
  英雄技能介绍

- @/docs/champions/{champion_name}/history.md
  开发历史，记录开发过程，最重要的是记录开发时的决策的背景、过程、局限性等

- @/docs/champions/{champion_name}/todo.md
  尚未解决的问题

- @/docs/champions/{champion_name}/feedback.md
  用户提出的需求、问题

英雄的配置、定义存放在 @/assets/characters/{champion_name}/config.ron 中

播放动画时，使用的动画名字必须与 @/assets/characters/{champion_name}/animations/skin{N}.ron 中的定义一致，区分大小写

你需要在 @/crates/lol_champions/src/{champion_name} 中写代码，包括测试
