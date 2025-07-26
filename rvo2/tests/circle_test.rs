use rvo2::*;
use std::f32::consts::PI;

#[test]
fn test_circle_agents_no_overlap() {
    // 创建RVO2模拟器
    let mut sim = RVOSimulatorWrapper::new();
    sim.set_time_step(0.05); // 保持较小的时间步长

    const NUM_AGENTS: usize = 12;
    const CIRCLE_RADIUS: f32 = 25.0; // 进一步增加大圆的半径（原来是20.0）
    const AGENT_RADIUS: f32 = 0.5; // 保持agent半径不变
    const MAX_STEPS: usize = 5000; // 增加最大步数
    const SAFETY_FACTOR: f32 = 3.5; // 进一步增加安全系数（原来是3.0）

    // 创建12个agents，均匀分布在一个圆上
    let mut agent_ids = Vec::new();
    for i in 0..NUM_AGENTS {
        let angle = 2.0 * PI * (i as f32) / (NUM_AGENTS as f32);
        let x = CIRCLE_RADIUS * angle.cos();
        let y = CIRCLE_RADIUS * angle.sin();

        // 添加agent
        let id = sim.add_agent(
            &[x, y],      // 初始位置
            25.0,         // 进一步增加邻居距离（原来是20.0）
            30,           // 增加最大邻居数（原来是25）
            12.0,         // 进一步增加时间视野（原来是10.0）
            6.0,          // 增加时间视野障碍物（原来是5.0）
            AGENT_RADIUS, // agent半径
            0.3,          // 进一步减小最大速度（原来是0.5）
            &[0.0, 0.0],  // 初始速度
        );
        agent_ids.push(id);

        // 设置目标位置（对面的位置）
        let target_x = -x;
        let target_y = -y;
        let dist = (target_x * target_x + target_y * target_y).sqrt();
        // 进一步减小期望速度
        sim.set_agent_pref_velocity(id, &[target_x / (dist * 5.0), target_y / (dist * 5.0)]);
    }

    // 运行模拟
    let mut step = 0;
    while step < MAX_STEPS {
        // 更新模拟
        sim.do_step();

        // 每100步显示一次进度
        if step % 100 == 0 {
            println!("Step {}/{}", step, MAX_STEPS);
        }

        // 检查所有agent之间是否有重叠
        for i in 0..NUM_AGENTS {
            let pos_i = sim.get_agent_position(agent_ids[i]);
            for j in (i + 1)..NUM_AGENTS {
                let pos_j = sim.get_agent_position(agent_ids[j]);

                // 计算两个agent之间的距离
                let dx = pos_i[0] - pos_j[0];
                let dy = pos_i[1] - pos_j[1];
                let distance = (dx * dx + dy * dy).sqrt();

                // 使用更大的安全余量
                assert!(
                    distance > AGENT_RADIUS * SAFETY_FACTOR,
                    "Agents {} and {} overlap at step {}! Distance: {}",
                    i,
                    j,
                    step,
                    distance
                );
            }
        }

        // 检查是否所有agent都接近目标位置
        let mut all_reached = true;
        let mut max_dist = 0.0;
        for i in 0..NUM_AGENTS {
            let pos = sim.get_agent_position(agent_ids[i]);
            let target_x = -CIRCLE_RADIUS * (2.0 * PI * (i as f32) / (NUM_AGENTS as f32)).cos();
            let target_y = -CIRCLE_RADIUS * (2.0 * PI * (i as f32) / (NUM_AGENTS as f32)).sin();

            let dx = pos[0] - target_x;
            let dy = pos[1] - target_y;
            let dist_to_target = (dx * dx + dy * dy).sqrt();

            max_dist = if dist_to_target > max_dist {
                dist_to_target
            } else {
                max_dist
            };

            if dist_to_target > AGENT_RADIUS {
                all_reached = false;
                break;
            }
        }

        // 每100步显示最大距离
        if step % 100 == 0 {
            println!("Max distance to target: {}", max_dist);
        }

        if all_reached {
            println!("All agents reached their targets at step {}", step);
            break;
        }

        step += 1;
    }

    assert!(
        step < MAX_STEPS,
        "Simulation did not complete within {} steps",
        MAX_STEPS
    );
}
