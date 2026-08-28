use winnow::Parser;
use winnow::combinator::{alt, delimited, opt, repeat};
use winnow::error::ContextError;

use crate::dsl::common::{PResult, ident, number_f32, number_usize, string_literal, symbol, ws};
use crate::env_spec::EnvTrainingParams;

/// 从 DSL 中解析出来的环境元数据块
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EnvMetaBlock {
    pub name: String,
    pub label: Option<String>,
    pub tag: Option<String>,
    pub description: Option<String>,
    pub num_agents: Option<usize>,
    pub params: Option<EnvTrainingParams>,
}

enum EnvField {
    Label(String),
    Tag(String),
    Description(String),
    NumAgents(usize),
    Params(EnvTrainingParams),
}

/// 解析 `env <Name> { ... }` 块
pub fn parse_env_meta_block<'i>(input: &mut &'i str) -> PResult<EnvMetaBlock, ContextError> {
    ws.parse_next(input)?;
    symbol("env").parse_next(input)?;

    let name = ident.parse_next(input)?;

    let fields: Vec<EnvField> =
        delimited(symbol("{"), repeat(0.., parse_env_field), symbol("}")).parse_next(input)?;

    let mut block = EnvMetaBlock {
        name,
        ..Default::default()
    };

    for field in fields {
        match field {
            EnvField::Label(s) => block.label = Some(s),
            EnvField::Tag(s) => block.tag = Some(s),
            EnvField::Description(s) => block.description = Some(s),
            EnvField::NumAgents(n) => block.num_agents = Some(n),
            EnvField::Params(p) => block.params = Some(p),
        }
    }

    Ok(block)
}

fn parse_env_field<'i>(input: &mut &'i str) -> PResult<EnvField, ContextError> {
    ws.parse_next(input)?;
    let key = ident.parse_next(input)?;
    let _ = opt(alt((symbol(":"), symbol("=")))).parse_next(input)?;

    let field = match key.as_str() {
        "label" => {
            let val = string_literal.parse_next(input)?;
            EnvField::Label(val)
        }
        "tag" => {
            let val = string_literal.parse_next(input)?;
            EnvField::Tag(val)
        }
        "description" => {
            let val = string_literal.parse_next(input)?;
            EnvField::Description(val)
        }
        "num_agents" | "agents" => {
            let val = number_usize.parse_next(input)?;
            EnvField::NumAgents(val)
        }
        "params" | "training_params" => {
            let params = parse_params_block.parse_next(input)?;
            EnvField::Params(params)
        }
        _ => {
            let _ = alt((string_literal.void(), number_f32.void())).parse_next(input)?;
            EnvField::Label(String::new())
        }
    };

    let _ = opt(symbol(";")).parse_next(input)?;
    Ok(field)
}

enum ParamField {
    Lr(f32),
    Gamma(f32),
    GaeLambda(f32),
    ClipEps(f32),
    PpoEpochs(usize),
    HiddenDim(usize),
    RolloutSteps(usize),
    TotalIterations(usize),
}

fn parse_params_block<'i>(input: &mut &'i str) -> PResult<EnvTrainingParams, ContextError> {
    let fields: Vec<ParamField> =
        delimited(symbol("{"), repeat(0.., parse_param_field), symbol("}")).parse_next(input)?;

    let mut params = EnvTrainingParams {
        lr: 3e-4,
        gamma: 0.99,
        gae_lambda: 0.95,
        clip_eps: 0.2,
        ppo_epochs: 8,
        hidden_dim: 64,
        rollout_steps_per_env: 160,
        total_iterations: 500,
    };

    for field in fields {
        match field {
            ParamField::Lr(v) => params.lr = v,
            ParamField::Gamma(v) => params.gamma = v,
            ParamField::GaeLambda(v) => params.gae_lambda = v,
            ParamField::ClipEps(v) => params.clip_eps = v,
            ParamField::PpoEpochs(v) => params.ppo_epochs = v,
            ParamField::HiddenDim(v) => params.hidden_dim = v,
            ParamField::RolloutSteps(v) => params.rollout_steps_per_env = v,
            ParamField::TotalIterations(v) => params.total_iterations = v,
        }
    }

    Ok(params)
}

fn parse_param_field<'i>(input: &mut &'i str) -> PResult<ParamField, ContextError> {
    ws.parse_next(input)?;
    let key = ident.parse_next(input)?;
    let _ = opt(alt((symbol(":"), symbol("=")))).parse_next(input)?;

    let field = match key.as_str() {
        "lr" | "learning_rate" => ParamField::Lr(number_f32.parse_next(input)?),
        "gamma" => ParamField::Gamma(number_f32.parse_next(input)?),
        "gae_lambda" | "lambda" => ParamField::GaeLambda(number_f32.parse_next(input)?),
        "clip_eps" | "clip" => ParamField::ClipEps(number_f32.parse_next(input)?),
        "ppo_epochs" | "epochs" => ParamField::PpoEpochs(number_usize.parse_next(input)?),
        "hidden_dim" | "hidden" => ParamField::HiddenDim(number_usize.parse_next(input)?),
        "rollout_steps_per_env" | "rollout_steps" | "rollout" => {
            ParamField::RolloutSteps(number_usize.parse_next(input)?)
        }
        "total_iterations" | "iterations" | "iters" => {
            ParamField::TotalIterations(number_usize.parse_next(input)?)
        }
        _ => {
            let _ = number_f32.parse_next(input)?;
            ParamField::Lr(0.0)
        }
    };

    let _ = opt(symbol(";")).parse_next(input)?;
    Ok(field)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_env_block() {
        let src = r#"
        env SoloV0 {
            label: "剑姬 vs 瑞雯 (Solo 1v1 自博弈)"
            tag: "SoloV0"
            description: "单神经网络通过 role_id (0:剑姬, 1:瑞雯) 自博弈对抗"
            num_agents: 2
            params {
                lr: 0.0003
                gamma: 0.99
                gae_lambda: 0.95
                clip_eps: 0.2
                ppo_epochs: 8
                hidden_dim: 64
                rollout_steps_per_env: 160
                total_iterations: 500
            }
        }
        "#;
        let mut input = src;
        let meta = parse_env_meta_block
            .parse_next(&mut input)
            .expect("meta parse ok");
        assert_eq!(meta.name, "SoloV0");
        assert_eq!(
            meta.label.as_deref(),
            Some("剑姬 vs 瑞雯 (Solo 1v1 自博弈)")
        );
        assert_eq!(meta.tag.as_deref(), Some("SoloV0"));
        assert_eq!(meta.num_agents, Some(2));
        let p = meta.params.expect("params ok");
        assert_eq!(p.lr, 0.0003);
        assert_eq!(p.rollout_steps_per_env, 160);
        assert_eq!(p.total_iterations, 500);
    }
}
