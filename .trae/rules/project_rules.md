# 回复

- 始终回复中文

# 代码规范

- 只添加必要的注释，避免添加冗余的注释
- 缩进不能超过 3 层，尽量使用 if return 或者 let else return 或者提取函数使代码扁平化

# bevy 0.16

- bevy 0.16 已经废弃了 camera bundle、mesh bundle 等 bundle，转而使用 require 来隐式创建必要的 bundle，所以避免使用 Camera3dBundle MeshBundle 等结构。