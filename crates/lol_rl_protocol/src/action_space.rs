use serde::{Deserialize, Serialize};

/// 强化学习动作空间描述，供训练/可视化循环区分离散与连续策略。
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum ActionSpace {
    /// 纯离散分类，动作数 n（legacy 环境）。
    Discrete(usize),
    /// 纯连续高斯，维度 d。
    Continuous(usize),
    /// 混合：连续高斯 d 维 + 一个离散分类 k 类。
    Hybrid {
        continuous_dims: usize,
        discrete_classes: usize,
    },
}

impl ActionSpace {
    /// Actor 头输出维度：Discrete(n)=n，Continuous(d)=d，Hybrid=d+k。
    pub fn actor_head_dim(&self) -> usize {
        match self {
            Self::Discrete(n) => *n,
            Self::Continuous(d) => *d,
            Self::Hybrid {
                continuous_dims,
                discrete_classes,
            } => continuous_dims + discrete_classes,
        }
    }

    /// Rollout 缓冲区中单个动作的扁平编码长度：
    /// Discrete=1（分类索引），Continuous=d，Hybrid=d+1（末位为攻击分类索引）。
    pub fn encoding_dim(&self) -> usize {
        match self {
            Self::Discrete(_) => 1,
            Self::Continuous(d) => *d,
            Self::Hybrid {
                continuous_dims, ..
            } => continuous_dims + 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_space_dims() {
        assert_eq!(ActionSpace::Discrete(5).actor_head_dim(), 5);
        assert_eq!(ActionSpace::Discrete(5).encoding_dim(), 1);
        assert_eq!(ActionSpace::Continuous(3).actor_head_dim(), 3);
        assert_eq!(ActionSpace::Continuous(3).encoding_dim(), 3);
        let hybrid = ActionSpace::Hybrid {
            continuous_dims: 2,
            discrete_classes: 2,
        };
        assert_eq!(hybrid.actor_head_dim(), 4);
        assert_eq!(hybrid.encoding_dim(), 3);
    }
}
