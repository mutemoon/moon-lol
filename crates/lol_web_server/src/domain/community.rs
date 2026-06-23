//! Community 子系统的领域层（Fork 关系 + 社区浏览）。
//!
//! Community 复用 AgentRepo / AgentConfigRepo，不新建表。
//! 这里只放纯逻辑：Fork 命名、社区可见性筛选。

use super::agent::fork_name;

/// 社区 Agent 排序维度。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommunitySort {
    Recent,
    Rating,
    Forks,
}

impl CommunitySort {
    pub fn as_str(&self) -> &'static str {
        match self {
            CommunitySort::Recent => "recent",
            CommunitySort::Rating => "rating",
            CommunitySort::Forks => "forks",
        }
    }
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "recent" => Some(CommunitySort::Recent),
            "rating" => Some(CommunitySort::Rating),
            "forks" => Some(CommunitySort::Forks),
            _ => None,
        }
    }
}

/// Fork 时新 Agent 的命名规则：若用户提供了 new_name 用之，否则用 fork_name 派生。
pub fn resolve_fork_name(provided: Option<&str>, original: &str) -> String {
    match provided {
        Some(name) => {
            let n = name.trim();
            if n.is_empty() {
                fork_name(original)
            } else {
                n.to_string()
            }
        }
        None => fork_name(original),
    }
}

/// 判断一个 Agent 是否可被 Fork（必须 Public 或 Friends，且不是自己的）。
pub fn can_fork(
    agent_visibility: super::spawn_preset::Visibility,
    owner_id: i32,
    requester_id: i32,
) -> bool {
    if owner_id == requester_id {
        return false; // 不能 Fork 自己的
    }
    matches!(
        agent_visibility,
        super::spawn_preset::Visibility::Public | super::spawn_preset::Visibility::Friends
    )
}

#[cfg(test)]
mod tests {
    use super::super::spawn_preset::Visibility;
    use super::*;

    #[test]
    fn resolve_fork_name_uses_provided() {
        assert_eq!(resolve_fork_name(Some("我的锐雯"), "原版"), "我的锐雯");
    }

    #[test]
    fn resolve_fork_name_derives_when_none() {
        assert_eq!(resolve_fork_name(None, "锐雯"), "锐雯 · 副本");
    }

    #[test]
    fn resolve_fork_name_derives_when_blank() {
        assert_eq!(resolve_fork_name(Some("   "), "锐雯"), "锐雯 · 副本");
    }

    #[test]
    fn can_fork_public_from_other() {
        assert!(can_fork(Visibility::Public, 1, 2));
    }

    #[test]
    fn cannot_fork_own_agent() {
        assert!(!can_fork(Visibility::Public, 1, 1));
    }

    #[test]
    fn cannot_fork_private_from_other() {
        assert!(!can_fork(Visibility::Private, 1, 2));
    }

    #[test]
    fn can_fork_friends_from_other() {
        assert!(can_fork(Visibility::Friends, 1, 2));
    }

    #[test]
    fn sort_roundtrip() {
        assert_eq!(
            CommunitySort::from_str("recent"),
            Some(CommunitySort::Recent)
        );
        assert_eq!(CommunitySort::from_str("forks"), Some(CommunitySort::Forks));
        assert_eq!(CommunitySort::from_str("x"), None);
    }
}
