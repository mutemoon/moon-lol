---
trigger: always_on
---

# 通用代码规范

1. 缩进不能超过 3 层，使用提前 return 减少缩进，避免多层 if 嵌套。
2. 如果缩进必须超过 3 层，说明该拆分为多个函数了。
3. 写注释要精简，不要写显而易见的注释。

# rust

1. let else return 或 if return 减少缩进
2. 要在文件顶部导入，不要用 qualify 导入，要用 use
3. debug! 时要写中文可读性强的日志，而不是使用冒号
