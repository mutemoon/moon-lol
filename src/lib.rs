pub mod buffs;
pub mod core;
pub mod entities;
pub mod server;
pub mod ui;

use core::action::PluginAction;
use core::aggro::PluginAggro;
use core::animation::PluginAnimation;
use core::attack::PluginAttack;
use core::attack_auto::PluginAttackAuto;
use core::base::state::PluginState;
use core::base::PluginBase;
use core::character::PluginCharacter;
use core::controller::PluginController;
use core::cooldown::PluginCooldown;
use core::damage::PluginDamage;
use core::game::PluginGame;
use core::life::PluginLife;
use core::map::PluginMap;
use core::missile::PluginMissile;
use core::movement::PluginMovement;
use core::navigation::navigation::PluginNavigaton;
use core::resource::PluginResource;
use core::rotate::PluginRotate;
use core::run::PluginRun;
use core::skill::PluginSkill;
use core::skin::PluginSkin;

use bevy::app::plugin_group;
use buffs::damage_reduction::PluginDamageReduction;
use buffs::fiora_e::PluginFioraE;
use buffs::fiora_passive::PluginFioraPassive;
use buffs::fiora_r::PluginFioraR;
use buffs::riven_passive::PluginRivenPassive;
use buffs::riven_q::PluginRivenQ;
use buffs::shield_magic::PluginShieldMagic;
use buffs::shield_white::PluginShieldWhite;
use entities::barrack::PluginBarrack;
use entities::champion::PluginChampion;
use entities::champions::aatrox::PluginAatrox;
use entities::champions::ahri::PluginAhri;
use entities::champions::akali::PluginAkali;
use entities::champions::alistar::PluginAlistar;
use entities::champions::amumu::PluginAmumu;
use entities::champions::anivia::PluginAnivia;
use entities::champions::annie::PluginAnnie;
use entities::champions::ashe::PluginAshe;
use entities::champions::aurora::PluginAurora;
use entities::champions::bard::PluginBard;
use entities::champions::blitzcrank::PluginBlitzcrank;
use entities::champions::brand::PluginBrand;
use entities::champions::braum::PluginBraum;
use entities::champions::caitlyn::PluginCaitlyn;
use entities::champions::camille::PluginCamille;
use entities::champions::cassiopeia::PluginCassiopeia;
use entities::champions::darius::PluginDarius;
use entities::champions::diana::PluginDiana;
use entities::champions::draven::PluginDraven;
use entities::champions::ekko::PluginEkko;
use entities::champions::evelynn::PluginEvelynn;
use entities::champions::ezreal::PluginEzreal;
use entities::champions::fiora::PluginFiora;
use entities::champions::fizz::PluginFizz;
use entities::champions::galio::PluginGalio;
use entities::champions::gangplank::PluginGangplank;
use entities::champions::garen::PluginGaren;
use entities::champions::gnar::PluginGnar;
use entities::champions::graves::PluginGraves;
use entities::champions::hecarim::PluginHecarim;
use entities::champions::heimerdinger::PluginHeimerdinger;
use entities::champions::hwei::PluginHwei;
use entities::champions::illaoi::PluginIllaoi;
use entities::champions::irelia::PluginIrelia;
use entities::champions::ivern::PluginIvern;
use entities::champions::janna::PluginJanna;
use entities::champions::jarvan::PluginJarvan;
use entities::champions::jax::PluginJax;
use entities::champions::jayce::PluginJayce;
use entities::champions::jinx::PluginJinx;
use entities::champions::kaisa::PluginKaisa;
use entities::champions::kalista::PluginKalista;
use entities::champions::karma::PluginKarma;
use entities::champions::katarina::PluginKatarina;
use entities::champions::kayle::PluginKayle;
use entities::champions::kayn::PluginKayn;
use entities::champions::kennen::PluginKennen;
use entities::champions::kindred::PluginKindred;
use entities::champions::kled::PluginKled;
use entities::champions::leblanc::PluginLeBlanc;
use entities::champions::leesin::PluginLeeSin;
use entities::champions::leona::PluginLeona;
use entities::champions::lissandra::PluginLissandra;
use entities::champions::lucian::PluginLucian;
use entities::champions::lulu::PluginLulu;
use entities::champions::lux::PluginLux;
use entities::champions::malzahar::PluginMalzahar;
use entities::champions::maokai::PluginMaokai;
use entities::champions::masteryi::PluginMasterYi;
use entities::champions::missfortune::PluginMissFortune;
use entities::champions::morgana::PluginMorgana;
use entities::champions::nami::PluginNami;
use entities::champions::nasus::PluginNasus;
use entities::champions::nautilus::PluginNautilus;
use entities::champions::neeko::PluginNeeko;
use entities::champions::nidalee::PluginNidalee;
use entities::champions::nocturne::PluginNocturne;
use entities::champions::olaf::PluginOlaf;
use entities::champions::orianna::PluginOrianna;
use entities::champions::ornn::PluginOrnn;
use entities::champions::pantheon::PluginPantheon;
use entities::champions::pyke::PluginPyke;
use entities::champions::qiyana::PluginQiyana;
use entities::champions::quinn::PluginQuinn;
use entities::champions::rakan::PluginRakan;
use entities::champions::rammus::PluginRammus;
use entities::champions::rell::PluginRell;
use entities::champions::renata::PluginRenata;
use entities::champions::renekton::PluginRenekton;
use entities::champions::rengar::PluginRengar;
use entities::champions::riven::PluginRiven;
use entities::champions::rumble::PluginRumble;
use entities::champions::ryze::PluginRyze;
use entities::champions::samira::PluginSamira;
use entities::champions::sejuani::PluginSejuani;
use entities::champions::senna::PluginSenna;
use entities::champions::seraphine::PluginSeraphine;
use entities::champions::sett::PluginSett;
use entities::champions::shaco::PluginShaco;
use entities::champions::shen::PluginShen;
use entities::champions::shyvana::PluginShyvana;
use entities::champions::singed::PluginSinged;
use entities::champions::sion::PluginSion;
use entities::champions::sivir::PluginSivir;
use entities::champions::skarner::PluginSkarner;
use entities::champions::smolder::PluginSmolder;
use entities::champions::sona::PluginSona;
use entities::champions::soraka::PluginSoraka;
use entities::champions::swain::PluginSwain;
use entities::champions::sylas::PluginSylas;
use entities::champions::syndra::PluginSyndra;
use entities::champions::tahm_kench::PluginTahmKench;
use entities::champions::taliyah::PluginTaliyah;
use entities::champions::talon::PluginTalon;
use entities::champions::taric::PluginTaric;
use entities::champions::teemo::PluginTeemo;
use entities::champions::thresh::PluginThresh;
use entities::champions::tristana::PluginTristana;
use entities::champions::trundle::PluginTrundle;
use entities::champions::tryndamere::PluginTryndamere;
use entities::champions::twisted_fate::PluginTwistedFate;
use entities::champions::twitch::PluginTwitch;
use entities::champions::urgot::PluginUrgot;
use entities::champions::volibear::PluginVolibear;
use entities::minion::PluginMinion;
use entities::shpere::PluginDebugSphere;
use entities::turret::PluginTurret;
use lol_core::lifetime::PluginLifetime;
use lol_core_render::camera::PluginCamera;
use lol_particle::PluginParticle;
use ui::PluginUI;

plugin_group! {
    pub struct PluginCore {
        :PluginDamageReduction,
        :PluginFioraPassive,
        :PluginFioraE,
        :PluginFioraR,
        :PluginRivenPassive,
        :PluginRivenQ,
        :PluginShieldWhite,
        :PluginShieldMagic,

        :PluginBarrack,
        :PluginChampion,
        :PluginCharacter,
        :PluginDebugSphere,
        :PluginMinion,
        :PluginTurret,

        :PluginAatrox,
        :PluginAhri,
        :PluginAkali,
        :PluginAlistar,
        :PluginAmumu,
        :PluginAnivia,
        :PluginAnnie,
        :PluginAshe,
        :PluginAurora,
        :PluginBard,
        :PluginBlitzcrank,
        :PluginBrand,
        :PluginBraum,
        :PluginCaitlyn,
        :PluginCamille,
        :PluginCassiopeia,
        :PluginDiana,
        :PluginDraven,
        :PluginEkko,
        :PluginEvelynn,
        :PluginEzreal,
        :PluginFizz,
        :PluginGalio,
        :PluginGangplank,
        :PluginGraves,
        :PluginHeimerdinger,
        :PluginIllaoi,
        :PluginIvern,
        :PluginJanna,
        :PluginJarvan,
        :PluginJayce,
        :PluginJinx,
        :PluginKaisa,
        :PluginKalista,
        :PluginKarma,
        :PluginKatarina,
        :PluginKayle,
        :PluginKennen,
        :PluginKindred,
        :PluginKled,
        :PluginLeBlanc,
        :PluginLeona,
        :PluginLissandra,
        :PluginLucian,
        :PluginLulu,
        :PluginLux,
        :PluginMalzahar,
        :PluginMaokai,
        :PluginMasterYi,
        :PluginMissFortune,
        :PluginMorgana,
        :PluginNami,
        :PluginNasus,
        :PluginNautilus,
        :PluginNeeko,
        :PluginNidalee,
        :PluginNocturne,
        :PluginOrianna,
        :PluginOrnn,
        :PluginPyke,
        :PluginQiyana,
        :PluginQuinn,
        :PluginRakan,
        :PluginRammus,
        :PluginRell,
        :PluginRenata,
        :PluginRengar,
        :PluginRumble,
        :PluginRyze,
        :PluginSamira,
        :PluginSejuani,
        :PluginSenna,
        :PluginSeraphine,
        :PluginShaco,
        :PluginShen,
        :PluginShyvana,
        :PluginSinged,
        :PluginSion,
        :PluginSivir,
        :PluginSkarner,
        :PluginSmolder,
        :PluginSona,
        :PluginSoraka,
        :PluginSwain,
        :PluginSyndra,
        :PluginTahmKench,
        :PluginTaliyah,
        :PluginTalon,
        :PluginTaric,
        :PluginTeemo,
        :PluginThresh,
        :PluginTristana,
        :PluginTrundle,
        :PluginTryndamere,
        :PluginTwistedFate,
        :PluginTwitch,
        :PluginDarius,
        :PluginFiora,
        :PluginGaren,
        :PluginGnar,
        :PluginHecarim,
        :PluginHwei,
        :PluginIrelia,
        :PluginJax,
        :PluginKayn,
        :PluginLeeSin,
        :PluginOlaf,
        :PluginPantheon,
        :PluginRenekton,
        :PluginRiven,
        :PluginSett,
        :PluginSylas,
        :PluginUrgot,
        :PluginVolibear,

        :PluginAction,
        :PluginAnimation,
        :PluginAttack,
        :PluginAttackAuto,
        :PluginAggro,
        :PluginBase,
        :PluginCamera,
        :PluginController,
        :PluginCooldown,
        :PluginDamage,
        :PluginGame,
        :PluginLife,
        :PluginLifetime,
        :PluginMap,
        :PluginMissile,
        :PluginMovement,
        :PluginNavigaton,
        :PluginParticle,
        :PluginResource,
        :PluginRotate,
        :PluginRun,
        :PluginSkill,
        :PluginSkin,
        :PluginState,
        :PluginUI,
    }
}
