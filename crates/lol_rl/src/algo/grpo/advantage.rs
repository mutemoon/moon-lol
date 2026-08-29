use crate::algo::buffer::RolloutBuffer;
use crate::algo::grpo::agent::GRPOAgent;

impl GRPOAgent {
    /// 计算 GRPO 分组相对优势 (Group Relative Advantages)
    ///
    /// 对于输入的若干环境轨迹 Buffers，按照 `group_size` 进行分组；
    /// 1. 对每个 buffer 计算其时间步上的未来折扣累计回报 $G_t = \sum_{k=t}^T \gamma^{k-t} r_k$；
    /// 2. 在每个组（Group）内，聚合所有轨迹的各步回报，计算组内均值 $\mu_G$ 与方差 $\sigma_G$；
    /// 3. 计算相对优势 $A_{i,t} = \frac{G_{i,t} - \mu_G}{\sigma_G + \epsilon}$；
    /// 4. 返回所有 buffers 对应的优势列表以及组统计回报（mean, std）。
    pub fn compute_group_advantages(
        &self,
        buffers: &[RolloutBuffer],
        group_size: usize,
    ) -> (Vec<Vec<f32>>, f32, f32) {
        let num_buffers = buffers.len();
        if num_buffers == 0 {
            return (Vec::new(), 0.0, 0.0);
        }

        let g_size = group_size.max(1);
        let mut all_advantages: Vec<Vec<f32>> = Vec::with_capacity(num_buffers);
        for b in buffers {
            all_advantages.push(vec![0.0; b.len()]);
        }

        let mut group_means = Vec::new();
        let mut group_stds = Vec::new();

        // 逐 Group 处理
        let mut start_idx = 0;
        while start_idx < num_buffers {
            let end_idx = (start_idx + g_size).min(num_buffers);
            let cur_group_buffers = &buffers[start_idx..end_idx];

            // 1. 计算当前组内每个 buffer 每步的未来折扣累积回报
            let mut group_discounted_returns: Vec<Vec<f32>> =
                Vec::with_capacity(cur_group_buffers.len());
            let mut all_returns_in_group = Vec::new();

            for buffer in cur_group_buffers {
                let n = buffer.len();
                let mut rets = vec![0.0; n];
                let mut run_ret = 0.0f32;
                for t in (0..n).rev() {
                    let done = buffer.dones.get(t).copied().unwrap_or(false);
                    let reward = buffer.rewards.get(t).copied().unwrap_or(0.0);
                    if done {
                        run_ret = reward;
                    } else {
                        run_ret = reward + self.config.gamma * run_ret;
                    }
                    rets[t] = run_ret;
                    all_returns_in_group.push(run_ret);
                }
                group_discounted_returns.push(rets);
            }

            // 2. 组内均值与标准差标准化
            let total_steps_in_group = all_returns_in_group.len();
            if total_steps_in_group > 0 {
                let mean = all_returns_in_group.iter().sum::<f32>() / (total_steps_in_group as f32);
                let variance = all_returns_in_group
                    .iter()
                    .map(|r| (r - mean).powi(2))
                    .sum::<f32>()
                    / (total_steps_in_group as f32);
                let std = (variance + 1e-8).sqrt();

                group_means.push(mean);
                group_stds.push(std);

                for (local_i, rets) in group_discounted_returns.iter().enumerate() {
                    let buf_idx = start_idx + local_i;
                    for (t, &val) in rets.iter().enumerate() {
                        all_advantages[buf_idx][t] = (val - mean) / std;
                    }
                }
            }

            start_idx += g_size;
        }

        let avg_mean = if group_means.is_empty() {
            0.0
        } else {
            group_means.iter().sum::<f32>() / (group_means.len() as f32)
        };
        let avg_std = if group_stds.is_empty() {
            0.0
        } else {
            group_stds.iter().sum::<f32>() / (group_stds.len() as f32)
        };

        (all_advantages, avg_mean, avg_std)
    }
}
