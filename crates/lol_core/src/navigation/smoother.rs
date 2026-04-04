use bevy::math::{vec2, Vec2};

fn rotate(v: Vec2) -> Vec2 {
    vec2(v.y, v.x)
}

/// 将网格路径转换为传送门（portal）列表
fn build_portals(points: &Vec<Vec2>) -> Vec<(Vec2, Vec2)> {
    let mut portals = Vec::new();
    if points.len() < 2 {
        return portals;
    }

    for i in 0..points.len() - 1 {
        let p1 = (points[i] - 0.5).round() + 0.5;
        let p2 = (points[i + 1] - 0.5).round() + 0.5;

        let p1f = rotate(p1);

        let dy = (p2.x - p1.x) as i32;
        let dx = (p2.y - p1.y) as i32;

        let (left, right) = match (dy, dx) {
            // 水平和垂直移动 (逻辑不变)
            (0, 1) => (
                rotate(vec2(p1f.y - 0.5, p1f.x + 0.5)),
                rotate(vec2(p1f.y + 0.5, p1f.x + 0.5)),
            ), // 右
            (0, -1) => (
                rotate(vec2(p1f.y + 0.5, p1f.x - 0.5)),
                rotate(vec2(p1f.y - 0.5, p1f.x - 0.5)),
            ), // 左
            (1, 0) => (
                rotate(vec2(p1f.y + 0.5, p1f.x + 0.5)),
                rotate(vec2(p1f.y + 0.5, p1f.x - 0.5)),
            ), // 下
            (-1, 0) => (
                rotate(vec2(p1f.y - 0.5, p1f.x - 0.5)),
                rotate(vec2(p1f.y - 0.5, p1f.x + 0.5)),
            ), // 上

            // 对角线移动 (新增逻辑)
            // 虚拟传送门的顶点是相邻“障碍”单元格的中心
            (1, 1) => (
                rotate(vec2(p1f.y + 0.5, p1f.x + 0.5)),
                rotate(vec2(p1f.y + 0.5, p1f.x + 0.5)),
            ), // 右下
            (1, -1) => (
                rotate(vec2(p1f.y + 0.5, p1f.x - 0.5)),
                rotate(vec2(p1f.y + 0.5, p1f.x - 0.5)),
            ), // 左下
            (-1, -1) => (
                rotate(vec2(p1f.y - 0.5, p1f.x - 0.5)),
                rotate(vec2(p1f.y - 0.5, p1f.x - 0.5)),
            ), // 左上
            (-1, 1) => (
                rotate(vec2(p1f.y - 0.5, p1f.x + 0.5)),
                rotate(vec2(p1f.y - 0.5, p1f.x + 0.5)),
            ), // 右上

            _ => {
                // 对于非连续路径（跳跃）或其他无效移动，此处会 panic
                panic!(
                    "Invalid or non-continuous movement detected in path: from {:?} to {:?}",
                    p1, p2
                );
            }
        };
        portals.push((left, right));
    }
    portals
}

/// 使用漏斗算法简化在2D网格上生成的路径。
///
/// # Arguments
/// * `points` - 一个表示路径的网格单元坐标 `(row, column)` 的向量。
///              路径必须是连续的，且只包含水平和垂直移动。
///
/// # Returns
/// 一个简化后的路径点向量 `(y, x)`。
pub fn simplify_path(points: &Vec<Vec2>) -> Vec<Vec2> {
    // 1. 处理边缘情况
    if points.len() < 3 {
        return points.clone();
    }

    let mut simplified_points = Vec::new();

    // 2. 将网格路径转换为传送门列表
    let mut portals = build_portals(points);

    // 3. 设置起点和终点
    let start_pos = rotate(points.first().unwrap().clone());
    let end_pos = rotate(points.last().unwrap().clone());

    // 添加一个零宽度的终点传送门，以确保算法能处理到路径末端
    portals.push((end_pos, end_pos));

    simplified_points.push(start_pos);

    // 4. 初始化漏斗
    let mut apex = start_pos;
    let (mut left_tentacle, mut right_tentacle) = portals[0];

    let mut i = 1;
    while i < portals.len() {
        let (new_left, new_right) = portals[i];

        // 5. 尝试更新右触手
        // 向量：apex -> right_tentacle
        let vec_r = right_tentacle - apex;
        // 向量：apex -> new_right
        let vec_nr = new_right - apex;

        // 如果 new_right 在 right_tentacle 的左侧或共线 (叉乘 <= 0),
        // 表示漏斗在右侧没有变宽。
        if vec_r.perp_dot(vec_nr) <= 0.0 {
            // 向量：apex -> left_tentacle
            let vec_l = left_tentacle - apex;
            // 检查 new_right 是否仍在 left_tentacle 的右侧
            if vec_l.perp_dot(vec_nr) >= 0.0 {
                // new_right 仍在漏斗内，收紧右触手
                right_tentacle = new_right;
            } else {
                // new_right 越过了左触手，说明我们找到了一个拐点 (left_tentacle)。
                // 将 left_tentacle 添加到路径中。
                simplified_points.push(left_tentacle);
                // 更新 apex 为这个新的拐点
                apex = left_tentacle;

                // 从这个拐点所在的传送门之后重新开始
                // 我们需要找到 left_tentacle 第一次出现为传送门顶点的索引
                let restart_idx = portals
                    .iter()
                    .position(|p| p.0 == left_tentacle || p.1 == left_tentacle)
                    .unwrap_or(i);
                i = restart_idx + 1;

                // 重置漏斗
                if i < portals.len() {
                    left_tentacle = portals[i].0;
                    right_tentacle = portals[i].1;
                }
                continue;
            }
        }

        // 6. 尝试更新左触手（与右侧对称）
        let vec_l = left_tentacle - apex;
        let vec_nl = new_left - apex;

        if vec_l.perp_dot(vec_nl) >= 0.0 {
            let vec_r = right_tentacle - apex;
            if vec_r.perp_dot(vec_nl) <= 0.0 {
                left_tentacle = new_left;
            } else {
                simplified_points.push(right_tentacle);
                apex = right_tentacle;

                let restart_idx = portals
                    .iter()
                    .position(|p| p.0 == right_tentacle || p.1 == right_tentacle)
                    .unwrap_or(i);
                i = restart_idx + 1;

                if i < portals.len() {
                    left_tentacle = portals[i].0;
                    right_tentacle = portals[i].1;
                }
                continue;
            }
        }
        i += 1;
    }

    // 7. 添加终点
    simplified_points.push(end_pos);

    // 8. 转换为用户期望的输出格式
    simplified_points.into_iter().map(|v| rotate(v)).collect()
}
