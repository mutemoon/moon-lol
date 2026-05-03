pub mod action;
pub mod aggro;
pub mod attack;
pub mod attack_auto;
pub mod base;
pub mod buffs;
pub mod character;
pub mod config;
pub mod cooldown;
pub mod damage;
pub mod effect;
pub mod entities;
pub mod error;
pub mod game;
pub mod lane;
pub mod life;
pub mod lifetime;
pub mod loaders;
pub mod log;
pub mod map;
pub mod missile;
pub mod movement;
pub mod navigation;
pub mod rotate;
pub mod run;
pub mod skill;
pub mod skill_script;
pub mod skin;
pub mod team;
pub mod utils;

use action::PluginAction;
use aggro::PluginAggro;
use attack::PluginAttack;
use attack_auto::PluginAttackAuto;
use base::PluginBase;
use base::state::PluginState;
use bevy::app::plugin_group;
use buffs::damage_reduction::PluginDamageReduction;
use buffs::shield_magic::PluginShieldMagic;
use buffs::shield_white::PluginShieldWhite;
use character::PluginCharacter;
use cooldown::PluginCooldown;
use damage::PluginDamage;
use entities::barrack::PluginBarrack;
use entities::champion::PluginChampion;
use entities::minion::PluginMinion;
use entities::shpere::PluginDebugSphere;
use entities::turret::PluginTurret;
use game::PluginGame;
use life::PluginLife;
use lifetime::PluginLifetime;
use map::PluginMap;
use missile::PluginMissile;
use movement::PluginMovement;
use navigation::navigation::PluginNavigaton;
use rotate::PluginRotate;
use run::PluginRun;
use skill::PluginSkill;
use skill_script::PluginSkillScript;

plugin_group! {
    pub struct PluginCore {
        :PluginAction,
        :PluginAggro,
        :PluginAttack,
        :PluginAttackAuto,
        :PluginBase,
        :PluginState,
        :PluginDamageReduction,
        :PluginShieldMagic,
        :PluginShieldWhite,
        :PluginCharacter,
        :PluginCooldown,
        :PluginDamage,
        :PluginBarrack,
        :PluginChampion,
        :PluginMinion,
        :PluginDebugSphere,
        :PluginTurret,
        :PluginGame,
        :PluginLife,
        :PluginLifetime,
        :PluginMap,
        :PluginMissile,
        :PluginMovement,
        :PluginNavigaton,
        :PluginRotate,
        :PluginRun,
        :PluginSkill,
        :PluginSkillScript,
    }
}
