# 回复风格

1. 始终使用中文回答

# bevy

0.16 有很多用法需要更新：

1. 不要使用 EventReader，而是使用 trigger: Trigger<CustomEvent>
2. 不要使用 commands.trigger_targets，而是使用 commands.entity(entity).trigger()
3. 不要使用 Camera3dBundle MeshBundle，而是使用 require 来隐式创建必要的 bundle

# 代码风格

1. 缩进不能超过 3 层，使用 if return 或者 let else return 或者提取函数减少缩进
2. 写注释要精简，不要写显而易见的注释

# 工程

1. 不要增加新的文档
2. 使用 cargo check 检查代码能否通过编译
