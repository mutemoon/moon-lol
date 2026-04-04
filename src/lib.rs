pub mod server;

use lol_core::action::PluginAction;
use lol_core::aggro::PluginAggro;
use lol_core::attack::PluginAttack;
use lol_core::attack_auto::PluginAttackAuto;
use lol_core::base::state::PluginState;
use lol_core::base::PluginBase;
use lol_core::character::PluginCharacter;
use lol_core::cooldown::PluginCooldown;
use lol_core::damage::PluginDamage;
use lol_core::game::PluginGame;
use lol_core::life::PluginLife;
use lol_core::map::PluginMap;
use lol_core::missile::PluginMissile;
use lol_core::movement::PluginMovement;
use lol_core::navigation::navigation::PluginNavigaton;
use lol_core::resource::PluginResource;
use lol_core::rotate::PluginRotate;
use lol_core::run::PluginRun;
use lol_core::skill::PluginSkill;
use lol_core_render::animation::PluginAnimation;
use lol_core_render::controller::PluginController;
use lol_core_render::skin::PluginSkin;

use bevy::app::plugin_group;
use lol_core::buffs::damage_reduction::PluginDamageReduction;
use lol_core::buffs::fiora_e::PluginFioraE;
use lol_core::buffs::fiora_passive::PluginFioraPassive;
use lol_core::buffs::fiora_r::PluginFioraR;
use lol_core::buffs::riven_passive::PluginRivenPassive;
use lol_core::buffs::riven_q::PluginRivenQ;
use lol_core::buffs::shield_magic::PluginShieldMagic;
use lol_core::buffs::shield_white::PluginShieldWhite;
use lol_core::entities::barrack::PluginBarrack;
use lol_core::entities::champion::PluginChampion;
use lol_core::entities::champions::aatrox::PluginAatrox;
use lol_core::entities::champions::ahri::PluginAhri;
use lol_core::entities::champions::akali::PluginAkali;
use lol_core::entities::champions::alistar::PluginAlistar;
use lol_core::entities::champions::amumu::PluginAmumu;
use lol_core::entities::champions::anivia::PluginAnivia;
use lol_core::entities::champions::annie::PluginAnnie;
use lol_core::entities::champions::ashe::PluginAshe;
use lol_core::entities::champions::aurora::PluginAurora;
use lol_core::entities::champions::bard::PluginBard;
use lol_core::entities::champions::blitzcrank::PluginBlitzcrank;
use lol_core::entities::champions::brand::PluginBrand;
use lol_core::entities::champions::braum::PluginBraum;
use lol_core::entities::champions::caitlyn::PluginCaitlyn;
use lol_core::entities::champions::camille::PluginCamille;
use lol_core::entities::champions::cassiopeia::PluginCassiopeia;
use lol_core::entities::champions::darius::PluginDarius;
use lol_core::entities::champions::diana::PluginDiana;
use lol_core::entities::champions::draven::PluginDraven;
use lol_core::entities::champions::ekko::PluginEkko;
use lol_core::entities::champions::evelynn::PluginEvelynn;
use lol_core::entities::champions::ezreal::PluginEzreal;
use lol_core::entities::champions::fiora::PluginFiora;
use lol_core::entities::champions::fizz::PluginFizz;
use lol_core::entities::champions::galio::PluginGalio;
use lol_core::entities::champions::gangplank::PluginGangplank;
use lol_core::entities::champions::garen::PluginGaren;
use lol_core::entities::champions::gnar::PluginGnar;
use lol_core::entities::champions::graves::PluginGraves;
use lol_core::entities::champions::hecarim::PluginHecarim;
use lol_core::entities::champions::heimerdinger::PluginHeimerdinger;
use lol_core::entities::champions::hwei::PluginHwei;
use lol_core::entities::champions::illaoi::PluginIllaoi;
use lol_core::entities::champions::irelia::PluginIrelia;
use lol_core::entities::champions::ivern::PluginIvern;
use lol_core::entities::champions::janna::PluginJanna;
use lol_core::entities::champions::jarvan::PluginJarvan;
use lol_core::entities::champions::jax::PluginJax;
use lol_core::entities::champions::jayce::PluginJayce;
use lol_core::entities::champions::jinx::PluginJinx;
use lol_core::entities::champions::kaisa::PluginKaisa;
use lol_core::entities::champions::kalista::PluginKalista;
use lol_core::entities::champions::karma::PluginKarma;
use lol_core::entities::champions::katarina::PluginKatarina;
use lol_core::entities::champions::kayle::PluginKayle;
use lol_core::entities::champions::kayn::PluginKayn;
use lol_core::entities::champions::kennen::PluginKennen;
use lol_core::entities::champions::kindred::PluginKindred;
use lol_core::entities::champions::kled::PluginKled;
use lol_core::entities::champions::leblanc::PluginLeBlanc;
use lol_core::entities::champions::leesin::PluginLeeSin;
use lol_core::entities::champions::leona::PluginLeona;
use lol_core::entities::champions::lissandra::PluginLissandra;
use lol_core::entities::champions::lucian::PluginLucian;
use lol_core::entities::champions::lulu::PluginLulu;
use lol_core::entities::champions::lux::PluginLux;
use lol_core::entities::champions::malzahar::PluginMalzahar;
use lol_core::entities::champions::maokai::PluginMaokai;
use lol_core::entities::champions::masteryi::PluginMasterYi;
use lol_core::entities::champions::missfortune::PluginMissFortune;
use lol_core::entities::champions::morgana::PluginMorgana;
use lol_core::entities::champions::nami::PluginNami;
use lol_core::entities::champions::nasus::PluginNasus;
use lol_core::entities::champions::nautilus::PluginNautilus;
use lol_core::entities::champions::neeko::PluginNeeko;
use lol_core::entities::champions::nidalee::PluginNidalee;
use lol_core::entities::champions::nocturne::PluginNocturne;
use lol_core::entities::champions::olaf::PluginOlaf;
use lol_core::entities::champions::orianna::PluginOrianna;
use lol_core::entities::champions::ornn::PluginOrnn;
use lol_core::entities::champions::pantheon::PluginPantheon;
use lol_core::entities::champions::pyke::PluginPyke;
use lol_core::entities::champions::qiyana::PluginQiyana;
use lol_core::entities::champions::quinn::PluginQuinn;
use lol_core::entities::champions::rakan::PluginRakan;
use lol_core::entities::champions::rammus::PluginRammus;
use lol_core::entities::champions::rell::PluginRell;
use lol_core::entities::champions::renata::PluginRenata;
use lol_core::entities::champions::renekton::PluginRenekton;
use lol_core::entities::champions::rengar::PluginRengar;
use lol_core::entities::champions::riven::PluginRiven;
use lol_core::entities::champions::rumble::PluginRumble;
use lol_core::entities::champions::ryze::PluginRyze;
use lol_core::entities::champions::samira::PluginSamira;
use lol_core::entities::champions::sejuani::PluginSejuani;
use lol_core::entities::champions::senna::PluginSenna;
use lol_core::entities::champions::seraphine::PluginSeraphine;
use lol_core::entities::champions::sett::PluginSett;
use lol_core::entities::champions::shaco::PluginShaco;
use lol_core::entities::champions::shen::PluginShen;
use lol_core::entities::champions::shyvana::PluginShyvana;
use lol_core::entities::champions::singed::PluginSinged;
use lol_core::entities::champions::sion::PluginSion;
use lol_core::entities::champions::sivir::PluginSivir;
use lol_core::entities::champions::skarner::PluginSkarner;
use lol_core::entities::champions::smolder::PluginSmolder;
use lol_core::entities::champions::sona::PluginSona;
use lol_core::entities::champions::soraka::PluginSoraka;
use lol_core::entities::champions::swain::PluginSwain;
use lol_core::entities::champions::sylas::PluginSylas;
use lol_core::entities::champions::syndra::PluginSyndra;
use lol_core::entities::champions::tahm_kench::PluginTahmKench;
use lol_core::entities::champions::taliyah::PluginTaliyah;
use lol_core::entities::champions::talon::PluginTalon;
use lol_core::entities::champions::taric::PluginTaric;
use lol_core::entities::champions::teemo::PluginTeemo;
use lol_core::entities::champions::thresh::PluginThresh;
use lol_core::entities::champions::tristana::PluginTristana;
use lol_core::entities::champions::trundle::PluginTrundle;
use lol_core::entities::champions::tryndamere::PluginTryndamere;
use lol_core::entities::champions::twisted_fate::PluginTwistedFate;
use lol_core::entities::champions::twitch::PluginTwitch;
use lol_core::entities::champions::urgot::PluginUrgot;
use lol_core::entities::champions::volibear::PluginVolibear;
use lol_core::entities::minion::PluginMinion;
use lol_core::entities::shpere::PluginDebugSphere;
use lol_core::entities::turret::PluginTurret;
use lol_core::lifetime::PluginLifetime;
use lol_core_render::camera::PluginCamera;
use lol_core_render::particle::PluginParticle;
use lol_core_render::ui::PluginUI;

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
