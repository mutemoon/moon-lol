pub mod server;

use bevy::app::plugin_group;
use lol_champions::buffs::fiora_e::PluginFioraE;
use lol_champions::buffs::fiora_passive::PluginFioraPassive;
use lol_champions::buffs::fiora_r::PluginFioraR;
use lol_champions::buffs::riven_passive::PluginRivenPassive;
use lol_champions::buffs::riven_q::PluginRivenQ;
use lol_champions::champions::aatrox::PluginAatrox;
use lol_champions::champions::ahri::PluginAhri;
use lol_champions::champions::akali::PluginAkali;
use lol_champions::champions::alistar::PluginAlistar;
use lol_champions::champions::amumu::PluginAmumu;
use lol_champions::champions::anivia::PluginAnivia;
use lol_champions::champions::annie::PluginAnnie;
use lol_champions::champions::ashe::PluginAshe;
use lol_champions::champions::aurora::PluginAurora;
use lol_champions::champions::bard::PluginBard;
use lol_champions::champions::blitzcrank::PluginBlitzcrank;
use lol_champions::champions::brand::PluginBrand;
use lol_champions::champions::braum::PluginBraum;
use lol_champions::champions::caitlyn::PluginCaitlyn;
use lol_champions::champions::camille::PluginCamille;
use lol_champions::champions::cassiopeia::PluginCassiopeia;
use lol_champions::champions::darius::PluginDarius;
use lol_champions::champions::diana::PluginDiana;
use lol_champions::champions::draven::PluginDraven;
use lol_champions::champions::ekko::PluginEkko;
use lol_champions::champions::evelynn::PluginEvelynn;
use lol_champions::champions::ezreal::PluginEzreal;
use lol_champions::champions::fiora::PluginFiora;
use lol_champions::champions::fizz::PluginFizz;
use lol_champions::champions::galio::PluginGalio;
use lol_champions::champions::gangplank::PluginGangplank;
use lol_champions::champions::garen::PluginGaren;
use lol_champions::champions::gnar::PluginGnar;
use lol_champions::champions::graves::PluginGraves;
use lol_champions::champions::hecarim::PluginHecarim;
use lol_champions::champions::heimerdinger::PluginHeimerdinger;
use lol_champions::champions::hwei::PluginHwei;
use lol_champions::champions::illaoi::PluginIllaoi;
use lol_champions::champions::irelia::PluginIrelia;
use lol_champions::champions::ivern::PluginIvern;
use lol_champions::champions::janna::PluginJanna;
use lol_champions::champions::jarvan::PluginJarvan;
use lol_champions::champions::jax::PluginJax;
use lol_champions::champions::jayce::PluginJayce;
use lol_champions::champions::jinx::PluginJinx;
use lol_champions::champions::kaisa::PluginKaisa;
use lol_champions::champions::kalista::PluginKalista;
use lol_champions::champions::karma::PluginKarma;
use lol_champions::champions::katarina::PluginKatarina;
use lol_champions::champions::kayle::PluginKayle;
use lol_champions::champions::kayn::PluginKayn;
use lol_champions::champions::kennen::PluginKennen;
use lol_champions::champions::kindred::PluginKindred;
use lol_champions::champions::kled::PluginKled;
use lol_champions::champions::leblanc::PluginLeBlanc;
use lol_champions::champions::leesin::PluginLeeSin;
use lol_champions::champions::leona::PluginLeona;
use lol_champions::champions::lissandra::PluginLissandra;
use lol_champions::champions::lucian::PluginLucian;
use lol_champions::champions::lulu::PluginLulu;
use lol_champions::champions::lux::PluginLux;
use lol_champions::champions::malzahar::PluginMalzahar;
use lol_champions::champions::maokai::PluginMaokai;
use lol_champions::champions::masteryi::PluginMasterYi;
use lol_champions::champions::missfortune::PluginMissFortune;
use lol_champions::champions::morgana::PluginMorgana;
use lol_champions::champions::nami::PluginNami;
use lol_champions::champions::nasus::PluginNasus;
use lol_champions::champions::nautilus::PluginNautilus;
use lol_champions::champions::neeko::PluginNeeko;
use lol_champions::champions::nidalee::PluginNidalee;
use lol_champions::champions::nocturne::PluginNocturne;
use lol_champions::champions::olaf::PluginOlaf;
use lol_champions::champions::orianna::PluginOrianna;
use lol_champions::champions::ornn::PluginOrnn;
use lol_champions::champions::pantheon::PluginPantheon;
use lol_champions::champions::pyke::PluginPyke;
use lol_champions::champions::qiyana::PluginQiyana;
use lol_champions::champions::quinn::PluginQuinn;
use lol_champions::champions::rakan::PluginRakan;
use lol_champions::champions::rammus::PluginRammus;
use lol_champions::champions::rell::PluginRell;
use lol_champions::champions::renata::PluginRenata;
use lol_champions::champions::renekton::PluginRenekton;
use lol_champions::champions::rengar::PluginRengar;
use lol_champions::champions::riven::PluginRiven;
use lol_champions::champions::rumble::PluginRumble;
use lol_champions::champions::ryze::PluginRyze;
use lol_champions::champions::samira::PluginSamira;
use lol_champions::champions::sejuani::PluginSejuani;
use lol_champions::champions::senna::PluginSenna;
use lol_champions::champions::seraphine::PluginSeraphine;
use lol_champions::champions::sett::PluginSett;
use lol_champions::champions::shaco::PluginShaco;
use lol_champions::champions::shen::PluginShen;
use lol_champions::champions::shyvana::PluginShyvana;
use lol_champions::champions::singed::PluginSinged;
use lol_champions::champions::sion::PluginSion;
use lol_champions::champions::sivir::PluginSivir;
use lol_champions::champions::skarner::PluginSkarner;
use lol_champions::champions::smolder::PluginSmolder;
use lol_champions::champions::sona::PluginSona;
use lol_champions::champions::soraka::PluginSoraka;
use lol_champions::champions::swain::PluginSwain;
use lol_champions::champions::sylas::PluginSylas;
use lol_champions::champions::syndra::PluginSyndra;
use lol_champions::champions::tahm_kench::PluginTahmKench;
use lol_champions::champions::taliyah::PluginTaliyah;
use lol_champions::champions::talon::PluginTalon;
use lol_champions::champions::taric::PluginTaric;
use lol_champions::champions::teemo::PluginTeemo;
use lol_champions::champions::thresh::PluginThresh;
use lol_champions::champions::tristana::PluginTristana;
use lol_champions::champions::trundle::PluginTrundle;
use lol_champions::champions::tryndamere::PluginTryndamere;
use lol_champions::champions::twisted_fate::PluginTwistedFate;
use lol_champions::champions::twitch::PluginTwitch;
use lol_champions::champions::urgot::PluginUrgot;
use lol_champions::champions::volibear::PluginVolibear;
use lol_core::action::PluginAction;
use lol_core::aggro::PluginAggro;
use lol_core::attack::PluginAttack;
use lol_core::attack_auto::PluginAttackAuto;
use lol_core::base::state::PluginState;
use lol_core::base::PluginBase;
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
use lol_core::resource::PluginResource;
use lol_core::rotate::PluginRotate;
use lol_core::run::PluginRun;
use lol_core::skill::PluginSkill;
use lol_render::animation::PluginAnimation;
use lol_render::camera::PluginCamera;
use lol_render::controller::PluginController;
use lol_render::particle::PluginParticle;
use lol_render::skin::PluginSkin;
use lol_render::ui::PluginUI;

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
