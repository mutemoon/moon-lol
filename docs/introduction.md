# 项目简介

关于技能系统的详细 ECS 架构与设计理念，请参考 [技能系统文档](skill.md)。

# bevy

0.17.0

# 游戏配置

从英雄联盟的 wad 文件中提取游戏资源

DATA/FINAL/Maps/Shipping/Map11.wad.client

这是目前主要读取的地图 wad 文件，存储了文件表和文件内容，

在文件表中记录了：

1. 每个文件的路径的 hash 值
2. 文件在 .wad.client 二进制数据中的 offset
3. 文件的 size
4. 其它信息

采用 u64 的 hash 存储路径，例如已知一个文件的路径为 data/maps/mapgeometry/map11/bloom.mapgeo，这个文件中存放了目前版本的地图中所有的模型。

就可以通过 hash 算法算出这个路径的 hash 值为 0xe8b4704f422901d9，这个 hash 算法是从 github 上开源的 LeagueToolkit 中获取的。

接下来就可以用 0xe8b4704f422901d9 找到这个文件的 offset 和 size，

然后就可以从 .wad.client 二进制数据中读取这个文件的内容了。

这些文件主要由模型文件、贴图文件、动画文件和配置文件组成，配置文件基本上都用 .bin 为后缀

兵营 barrack

# 左性坐标系 -> 右手坐标系

英雄联盟是左手坐标系，而 bevy 是右手坐标系，所以需要将摄像机的屏幕裁剪空间水平翻转。
