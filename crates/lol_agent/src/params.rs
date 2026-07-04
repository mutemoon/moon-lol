use lol_core::action::Action;
use serde::Deserialize;
use serde_json::Value;

#[derive(Deserialize, Debug, Clone)]
pub struct ObserveParams {
    pub entity_id: Option<u64>,
    #[serde(default)]
    pub json: bool,
}

#[derive(Debug, Clone)]
pub struct ActionParams {
    pub entity_id: Option<u64>,
    pub action: Action,
}

impl<'de> Deserialize<'de> for ActionParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct ActionWithEntity {
            entity_id: Option<u64>,
            action: Action,
        }

        let value = Value::deserialize(deserializer)?;
        if let Ok(wrapper) = serde_json::from_value::<ActionWithEntity>(value.clone()) {
            Ok(ActionParams {
                entity_id: wrapper.entity_id,
                action: wrapper.action,
            })
        } else if let Ok(action) = serde_json::from_value::<Action>(value) {
            Ok(ActionParams {
                entity_id: None,
                action,
            })
        } else {
            Err(serde::de::Error::custom("Invalid action params"))
        }
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct SetScriptParams {
    pub entity_id: u64,
    pub source: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RlResetParams {
    pub entity_id: Option<u64>,
    pub config_json: Option<Value>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct RlStepParams {
    pub entity_id: Option<u64>,
}

#[derive(Debug, Clone, Default)]
pub struct GetAgentsParams;

impl<'de> Deserialize<'de> for GetAgentsParams {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let _ = Value::deserialize(deserializer)?;
        Ok(GetAgentsParams)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_params_deserialization() {
        use bevy::prelude::Vec2;

        // 扁平 action
        let json_flat = serde_json::json!({
            "Move": [10.0, 20.0]
        });
        let params: ActionParams = serde_json::from_value(json_flat).unwrap();
        assert_eq!(params.entity_id, None);
        match params.action {
            Action::Move(pos) => assert_eq!(pos, Vec2::new(10.0, 20.0)),
            _ => panic!("Expected Action::Move"),
        }

        // 带 entity_id 的包装 action
        let json_wrapped = serde_json::json!({
            "entity_id": 123,
            "action": {
                "Move": [10.0, 20.0]
            }
        });
        let params: ActionParams = serde_json::from_value(json_wrapped).unwrap();
        assert_eq!(params.entity_id, Some(123));
        match params.action {
            Action::Move(pos) => assert_eq!(pos, Vec2::new(10.0, 20.0)),
            _ => panic!("Expected Action::Move"),
        }
    }
}
