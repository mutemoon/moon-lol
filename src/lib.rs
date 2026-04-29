use bevy::app::plugin_group;
use lol_champions::PluginChampions;
use lol_core::action::PluginAction;
use lol_core::aggro::PluginAggro;
use lol_core::attack::PluginAttack;
use lol_core::attack_auto::PluginAttackAuto;
use lol_core::base::PluginBase;
use lol_core::base::state::PluginState;
use lol_core::buffs::damage_reduction::PluginDamageReduction;
use lol_core::buffs::shield_magic::PluginShieldMagic;
use lol_core::buffs::shield_white::PluginShieldWhite;
use lol_core::character::PluginCharacter;
use lol_core::cooldown::PluginCooldown;
use lol_core::damage::PluginDamage;
use lol_core::entities::barrack::PluginBarrack;
use lol_core::entities::champion::PluginChampion;
use lol_core::entities::minion::PluginMinion;
use lol_core::entities::shpere::PluginDebugSphere;
use lol_core::entities::turret::PluginTurret;
use lol_core::game::PluginGame;
use lol_core::life::PluginLife;
use lol_core::lifetime::PluginLifetime;
use lol_core::map::PluginMap;
use lol_core::missile::PluginMissile;
use lol_core::movement::PluginMovement;
use lol_core::navigation::navigation::PluginNavigaton;
use lol_core::rotate::PluginRotate;
use lol_core::run::PluginRun;
use lol_core::skill::PluginSkill;
use lol_core::skill_script::PluginSkillScript;
use lol_render::PluginRender;

plugin_group! {
    pub struct PluginCore {
        :PluginChampions,

        :PluginDamageReduction,
        :PluginShieldWhite,
        :PluginShieldMagic,

        :PluginBarrack,
        :PluginChampion,
        :PluginCharacter,
        :PluginDebugSphere,
        :PluginMinion,
        :PluginTurret,

        :PluginAction,
        :PluginAttack,
        :PluginAttackAuto,
        :PluginAggro,
        :PluginBase,
        :PluginCooldown,
        :PluginDamage,
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
        :PluginState,

        :PluginSkillScript,

        :PluginRender,
    }
}
