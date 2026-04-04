pub mod server;

use bevy::app::plugin_group;
use lol_champions::aatrox::PluginAatrox;
use lol_champions::ahri::PluginAhri;
use lol_champions::akali::PluginAkali;
use lol_champions::alistar::PluginAlistar;
use lol_champions::amumu::PluginAmumu;
use lol_champions::anivia::PluginAnivia;
use lol_champions::annie::PluginAnnie;
use lol_champions::ashe::PluginAshe;
use lol_champions::aurora::PluginAurora;
use lol_champions::bard::PluginBard;
use lol_champions::blitzcrank::PluginBlitzcrank;
use lol_champions::brand::PluginBrand;
use lol_champions::braum::PluginBraum;
use lol_champions::caitlyn::PluginCaitlyn;
use lol_champions::camille::PluginCamille;
use lol_champions::cassiopeia::PluginCassiopeia;
use lol_champions::darius::PluginDarius;
use lol_champions::diana::PluginDiana;
use lol_champions::draven::PluginDraven;
use lol_champions::ekko::PluginEkko;
use lol_champions::evelynn::PluginEvelynn;
use lol_champions::ezreal::PluginEzreal;
use lol_champions::fiora::e::PluginFioraE;
use lol_champions::fiora::passive::PluginFioraPassive;
use lol_champions::fiora::r::PluginFioraR;
use lol_champions::fiora::PluginFiora;
use lol_champions::fizz::PluginFizz;
use lol_champions::galio::PluginGalio;
use lol_champions::gangplank::PluginGangplank;
use lol_champions::garen::PluginGaren;
use lol_champions::gnar::PluginGnar;
use lol_champions::graves::PluginGraves;
use lol_champions::hecarim::PluginHecarim;
use lol_champions::heimerdinger::PluginHeimerdinger;
use lol_champions::hwei::PluginHwei;
use lol_champions::illaoi::PluginIllaoi;
use lol_champions::irelia::PluginIrelia;
use lol_champions::ivern::PluginIvern;
use lol_champions::janna::PluginJanna;
use lol_champions::jarvan::PluginJarvan;
use lol_champions::jax::PluginJax;
use lol_champions::jayce::PluginJayce;
use lol_champions::jinx::PluginJinx;
use lol_champions::kaisa::PluginKaisa;
use lol_champions::kalista::PluginKalista;
use lol_champions::karma::PluginKarma;
use lol_champions::katarina::PluginKatarina;
use lol_champions::kayle::PluginKayle;
use lol_champions::kayn::PluginKayn;
use lol_champions::kennen::PluginKennen;
use lol_champions::kindred::PluginKindred;
use lol_champions::kled::PluginKled;
use lol_champions::leblanc::PluginLeBlanc;
use lol_champions::leesin::PluginLeeSin;
use lol_champions::leona::PluginLeona;
use lol_champions::lissandra::PluginLissandra;
use lol_champions::lucian::PluginLucian;
use lol_champions::lulu::PluginLulu;
use lol_champions::lux::PluginLux;
use lol_champions::malzahar::PluginMalzahar;
use lol_champions::maokai::PluginMaokai;
use lol_champions::masteryi::PluginMasterYi;
use lol_champions::missfortune::PluginMissFortune;
use lol_champions::morgana::PluginMorgana;
use lol_champions::nami::PluginNami;
use lol_champions::nasus::PluginNasus;
use lol_champions::nautilus::PluginNautilus;
use lol_champions::neeko::PluginNeeko;
use lol_champions::nidalee::PluginNidalee;
use lol_champions::nocturne::PluginNocturne;
use lol_champions::olaf::PluginOlaf;
use lol_champions::orianna::PluginOrianna;
use lol_champions::ornn::PluginOrnn;
use lol_champions::pantheon::PluginPantheon;
use lol_champions::pyke::PluginPyke;
use lol_champions::qiyana::PluginQiyana;
use lol_champions::quinn::PluginQuinn;
use lol_champions::rakan::PluginRakan;
use lol_champions::rammus::PluginRammus;
use lol_champions::rell::PluginRell;
use lol_champions::renata::PluginRenata;
use lol_champions::renekton::PluginRenekton;
use lol_champions::rengar::PluginRengar;
use lol_champions::riven::passive::PluginRivenPassive;
use lol_champions::riven::q::PluginRivenQ;
use lol_champions::riven::PluginRiven;
use lol_champions::rumble::PluginRumble;
use lol_champions::ryze::PluginRyze;
use lol_champions::samira::PluginSamira;
use lol_champions::sejuani::PluginSejuani;
use lol_champions::senna::PluginSenna;
use lol_champions::seraphine::PluginSeraphine;
use lol_champions::sett::PluginSett;
use lol_champions::shaco::PluginShaco;
use lol_champions::shen::PluginShen;
use lol_champions::shyvana::PluginShyvana;
use lol_champions::singed::PluginSinged;
use lol_champions::sion::PluginSion;
use lol_champions::sivir::PluginSivir;
use lol_champions::skarner::PluginSkarner;
use lol_champions::smolder::PluginSmolder;
use lol_champions::sona::PluginSona;
use lol_champions::soraka::PluginSoraka;
use lol_champions::swain::PluginSwain;
use lol_champions::sylas::PluginSylas;
use lol_champions::syndra::PluginSyndra;
use lol_champions::tahm_kench::PluginTahmKench;
use lol_champions::taliyah::PluginTaliyah;
use lol_champions::talon::PluginTalon;
use lol_champions::taric::PluginTaric;
use lol_champions::teemo::PluginTeemo;
use lol_champions::thresh::PluginThresh;
use lol_champions::tristana::PluginTristana;
use lol_champions::trundle::PluginTrundle;
use lol_champions::tryndamere::PluginTryndamere;
use lol_champions::twisted_fate::PluginTwistedFate;
use lol_champions::twitch::PluginTwitch;
use lol_champions::urgot::PluginUrgot;
use lol_champions::volibear::PluginVolibear;
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
use lol_render::map::PluginRenderMap;
use lol_render::particle::PluginParticle;
use lol_render::resource::PluginRenderResource;
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

        :PluginRenderMap,
        :PluginRenderResource,
    }
}
