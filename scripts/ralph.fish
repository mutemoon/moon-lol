#!/usr/bin/env fish

# 设置初始迭代次数
set iteration 0

while true
    # 计数器递增
    set iteration (math $iteration + 1)

    # 使用 fish 风格的日期输出和加粗文本
    set_color --bold cyan
    echo "=== 迭代 $iteration ==="
    set_color normal

    # 执行迁移指令
    # 使用 cat 读取文档并管道传输给 ccv-claude
    cat docs/MIGRATION_PLAN.md | ccv-claude -p --dangerously-skip-permissions

    # 稍微停顿，方便 Ctrl+C 中断
    sleep 1
end
