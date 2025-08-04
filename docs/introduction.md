bevy
0.16.1

直接从英雄联盟的二进制数据中读取游戏配置

DATA/FINAL/Maps/Shipping/Map11.wad.client

这是目前主要读取的地图文件，存储了文件表和文件内容，

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
