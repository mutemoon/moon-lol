use bevy::prelude::*;
use bevy_behave::{
    prelude::{BehaveCtx, BehavePlugin, BehaveTree, Tree},
    Behave,
};

#[derive(Default)]
pub struct PluginSkill;

impl Plugin for PluginSkill {
    fn build(&self, app: &mut App) {
        app.add_event::<CommandSkillStart>();

        app.add_plugins(BehavePlugin::default());

        app.add_observer(on_skill_cast);
    }
}

#[derive(Component, Debug)]
#[relationship(relationship_target = Skills)]
pub struct SkillOf(pub Entity);

#[derive(Component, Debug)]
#[relationship_target(relationship = SkillOf)]
pub struct Skills(Vec<Entity>);

#[derive(Component, Default)]
pub struct CoolDown {
    pub timer: Timer,
}

#[derive(Component)]
#[require(CoolDown)]
pub struct Skill {
    pub effect: Option<Tree<Behave>>,
}

#[derive(Component)]
pub struct SkillEffectContext {
    pub point: Vec2,
}

#[derive(Event)]
pub struct CommandSkillStart {
    pub index: usize,
    pub point: Vec2,
}

fn on_skill_cast(
    trigger: Trigger<CommandSkillStart>,
    mut commands: Commands,
    skills: Query<&Skills>,
    q_skill: Query<&Skill>,
) {
    let entity = trigger.target();
    let event = trigger.event();
    let skills = skills.get(entity).unwrap();
    let skill_entity = skills.0[event.index];
    let skill = q_skill.get(skill_entity).unwrap();

    if let Some(effect) = &skill.effect {
        commands.entity(entity).with_child((
            BehaveTree::new(effect.clone()),
            SkillEffectContext { point: event.point },
        ));
    }
}

#[derive(Component)]
pub struct SkillEffectBehaveCtx(pub BehaveCtx);
