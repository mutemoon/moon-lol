use bevy::app::Plugin;
use bevy::prelude::*;
use lol_core::base::ability_resource::{AbilityResource, AbilityResourceType};
use lol_core::base::level::Level;
use lol_core::damage::{Armor, Damage};
use lol_core::debug::handlers::ChampionSwitchQueue;
use lol_core::life::Health;
use lol_core::movement::{Movement, MovementState};
use lol_core::skill::SkillPoints;
use lol_core::team::Team;

pub mod aatrox;
pub mod ahri;
pub mod akali;
pub mod akshan;
pub mod alistar;
pub mod amumu;
pub mod anivia;
pub mod annie;
pub mod aphelios;
pub mod ashe;
pub mod aurora;
pub mod bard;
pub mod belveth;
pub mod blitzcrank;
pub mod brand;
pub mod braum;
pub mod briar;
pub mod caitlyn;
pub mod camille;
pub mod cassiopeia;
pub mod darius;
pub mod diana;
pub mod draven;
pub mod ekko;
pub mod evelynn;
pub mod ezreal;
pub mod fiora;
pub mod fizz;
pub mod galio;
pub mod gangplank;
pub mod garen;
pub mod gnar;
pub mod graves;
pub mod hecarim;
pub mod heimerdinger;
pub mod hwei;
pub mod illaoi;
pub mod irelia;
pub mod ivern;
pub mod janna;
pub mod jarvan;
pub mod jax;
pub mod jayce;
pub mod jinx;
pub mod kaisa;
pub mod kalista;
pub mod karma;
pub mod katarina;
pub mod kayle;
pub mod kayn;
pub mod kennen;
pub mod kindred;
pub mod kled;
pub mod leblanc;
pub mod leesin;
pub mod leona;
pub mod lissandra;
pub mod lucian;
pub mod lulu;
pub mod lux;
pub mod malzahar;
pub mod maokai;
pub mod masteryi;
pub mod missfortune;
pub mod morgana;
pub mod nami;
pub mod nasus;
pub mod nautilus;
pub mod neeko;
pub mod nidalee;
pub mod nocturne;
pub mod olaf;
pub mod orianna;
pub mod ornn;
pub mod pantheon;
pub mod pyke;
pub mod qiyana;
pub mod quinn;
pub mod rakan;
pub mod rammus;
pub mod rell;
pub mod renata;
pub mod renekton;
pub mod rengar;
pub mod riven;
pub mod rumble;
pub mod ryze;
pub mod samira;
pub mod sejuani;
pub mod senna;
pub mod seraphine;
pub mod sett;
pub mod shaco;
pub mod shen;
pub mod shyvana;
pub mod singed;
pub mod sion;
pub mod sivir;
pub mod skarner;
pub mod smolder;
pub mod sona;
pub mod soraka;
pub mod swain;
pub mod sylas;
pub mod syndra;
pub mod tahm_kench;
pub mod taliyah;
pub mod talon;
pub mod taric;
pub mod teemo;
pub mod thresh;
pub mod tristana;
pub mod trundle;
pub mod tryndamere;
pub mod twisted_fate;
pub mod twitch;
pub mod urgot;
pub mod volibear;

#[cfg(test)]
mod test_utils;

#[derive(Default)]
pub struct PluginChampions;

impl Plugin for PluginChampions {
    fn build(&self, app: &mut bevy::app::App) {
        app.add_systems(Update, process_champion_switch_queue);

        app.add_plugins(aatrox::PluginAatrox);
        app.add_plugins(ahri::PluginAhri);
        app.add_plugins(akali::PluginAkali);
        app.add_plugins(akshan::PluginAkshan);
        app.add_plugins(alistar::PluginAlistar);
        app.add_plugins(amumu::PluginAmumu);
        app.add_plugins(anivia::PluginAnivia);
        app.add_plugins(annie::PluginAnnie);
        app.add_plugins(aphelios::PluginAphelios);
        app.add_plugins(ashe::PluginAshe);
        app.add_plugins(aurora::PluginAurora);
        app.add_plugins(bard::PluginBard);
        app.add_plugins(belveth::PluginBelveth);
        app.add_plugins(blitzcrank::PluginBlitzcrank);
        app.add_plugins(brand::PluginBrand);
        app.add_plugins(braum::PluginBraum);
        app.add_plugins(briar::PluginBriar);
        app.add_plugins(caitlyn::PluginCaitlyn);
        app.add_plugins(camille::PluginCamille);
        app.add_plugins(cassiopeia::PluginCassiopeia);
        app.add_plugins(darius::PluginDarius);
        app.add_plugins(diana::PluginDiana);
        app.add_plugins(draven::PluginDraven);
        app.add_plugins(ekko::PluginEkko);
        app.add_plugins(evelynn::PluginEvelynn);
        app.add_plugins(ezreal::PluginEzreal);
        app.add_plugins(fiora::PluginFiora);
        app.add_plugins(fizz::PluginFizz);
        app.add_plugins(galio::PluginGalio);
        app.add_plugins(gangplank::PluginGangplank);
        app.add_plugins(garen::PluginGaren);
        app.add_plugins(gnar::PluginGnar);
        app.add_plugins(graves::PluginGraves);
        app.add_plugins(hecarim::PluginHecarim);
        app.add_plugins(heimerdinger::PluginHeimerdinger);
        app.add_plugins(hwei::PluginHwei);
        app.add_plugins(illaoi::PluginIllaoi);
        app.add_plugins(irelia::PluginIrelia);
        app.add_plugins(ivern::PluginIvern);
        app.add_plugins(janna::PluginJanna);
        app.add_plugins(jarvan::PluginJarvan);
        app.add_plugins(jax::PluginJax);
        app.add_plugins(jayce::PluginJayce);
        app.add_plugins(jinx::PluginJinx);
        app.add_plugins(kaisa::PluginKaisa);
        app.add_plugins(kalista::PluginKalista);
        app.add_plugins(karma::PluginKarma);
        app.add_plugins(katarina::PluginKatarina);
        app.add_plugins(kayle::PluginKayle);
        app.add_plugins(kayn::PluginKayn);
        app.add_plugins(kennen::PluginKennen);
        app.add_plugins(kindred::PluginKindred);
        app.add_plugins(kled::PluginKled);
        app.add_plugins(leblanc::PluginLeBlanc);
        app.add_plugins(leesin::PluginLeeSin);
        app.add_plugins(leona::PluginLeona);
        app.add_plugins(lissandra::PluginLissandra);
        app.add_plugins(lucian::PluginLucian);
        app.add_plugins(lulu::PluginLulu);
        app.add_plugins(lux::PluginLux);
        app.add_plugins(malzahar::PluginMalzahar);
        app.add_plugins(maokai::PluginMaokai);
        app.add_plugins(masteryi::PluginMasterYi);
        app.add_plugins(missfortune::PluginMissFortune);
        app.add_plugins(morgana::PluginMorgana);
        app.add_plugins(nami::PluginNami);
        app.add_plugins(nasus::PluginNasus);
        app.add_plugins(nautilus::PluginNautilus);
        app.add_plugins(neeko::PluginNeeko);
        app.add_plugins(nidalee::PluginNidalee);
        app.add_plugins(nocturne::PluginNocturne);
        app.add_plugins(olaf::PluginOlaf);
        app.add_plugins(orianna::PluginOrianna);
        app.add_plugins(ornn::PluginOrnn);
        app.add_plugins(pantheon::PluginPantheon);
        app.add_plugins(pyke::PluginPyke);
        app.add_plugins(qiyana::PluginQiyana);
        app.add_plugins(quinn::PluginQuinn);
        app.add_plugins(rakan::PluginRakan);
        app.add_plugins(rammus::PluginRammus);
        app.add_plugins(rell::PluginRell);
        app.add_plugins(renata::PluginRenata);
        app.add_plugins(renekton::PluginRenekton);
        app.add_plugins(rengar::PluginRengar);
        app.add_plugins(riven::PluginRiven);
        app.add_plugins(rumble::PluginRumble);
        app.add_plugins(ryze::PluginRyze);
        app.add_plugins(samira::PluginSamira);
        app.add_plugins(sejuani::PluginSejuani);
        app.add_plugins(senna::PluginSenna);
        app.add_plugins(seraphine::PluginSeraphine);
        app.add_plugins(sett::PluginSett);
        app.add_plugins(shaco::PluginShaco);
        app.add_plugins(shen::PluginShen);
        app.add_plugins(shyvana::PluginShyvana);
        app.add_plugins(singed::PluginSinged);
        app.add_plugins(sion::PluginSion);
        app.add_plugins(sivir::PluginSivir);
        app.add_plugins(skarner::PluginSkarner);
        app.add_plugins(smolder::PluginSmolder);
        app.add_plugins(sona::PluginSona);
        app.add_plugins(soraka::PluginSoraka);
        app.add_plugins(swain::PluginSwain);
        app.add_plugins(sylas::PluginSylas);
        app.add_plugins(syndra::PluginSyndra);
        app.add_plugins(tahm_kench::PluginTahmKench);
        app.add_plugins(taliyah::PluginTaliyah);
        app.add_plugins(talon::PluginTalon);
        app.add_plugins(taric::PluginTaric);
        app.add_plugins(teemo::PluginTeemo);
        app.add_plugins(thresh::PluginThresh);
        app.add_plugins(tristana::PluginTristana);
        app.add_plugins(trundle::PluginTrundle);
        app.add_plugins(tryndamere::PluginTryndamere);
        app.add_plugins(twisted_fate::PluginTwistedFate);
        app.add_plugins(twitch::PluginTwitch);
        app.add_plugins(urgot::PluginUrgot);
        app.add_plugins(volibear::PluginVolibear);
    }
}

/// Processes the ChampionSwitchQueue: spawns new champion entities when a switch is requested.
fn process_champion_switch_queue(mut commands: Commands, mut queue: ResMut<ChampionSwitchQueue>) {
    for name in queue.0.drain(..) {
        match name.as_str() {
            "Riven" => {
                commands.spawn((
                    crate::riven::Riven,
                    Team::Order,
                    Transform::default(),
                    Health::new(1000.0),
                    AbilityResource {
                        ar_type: AbilityResourceType::Mana,
                        value: 1000.0,
                        max: 1000.0,
                        base: 1000.0,
                        per_level: 0.0,
                        base_static_regen: 0.0,
                        regen_per_level: 0.0,
                    },
                    Level {
                        value: 18,
                        ..default()
                    },
                    SkillPoints(4),
                    Damage(100.0),
                    Armor(0.0),
                    Movement { speed: 340.0 },
                    MovementState::default(),
                ));
            }
            "Fiora" => {
                commands.spawn((
                    crate::fiora::Fiora,
                    Team::Order,
                    Transform::default(),
                    Health::new(1000.0),
                    AbilityResource {
                        ar_type: AbilityResourceType::Mana,
                        value: 1000.0,
                        max: 1000.0,
                        base: 1000.0,
                        per_level: 0.0,
                        base_static_regen: 0.0,
                        regen_per_level: 0.0,
                    },
                    Level {
                        value: 18,
                        ..default()
                    },
                    SkillPoints(4),
                    Damage(100.0),
                    Armor(0.0),
                    Movement { speed: 340.0 },
                    MovementState::default(),
                ));
            }
            _ => {
                warn!(target: "debug", "unknown champion: {}", name);
            }
        }
    }
}
