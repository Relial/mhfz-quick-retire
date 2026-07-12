use crate::{config::DeathRetireKind, plugin::STATE};

static mut SKIPPING: bool = false;

pub unsafe extern "C" fn on_quest_update() {
    unsafe {
        SKIPPING = false;

        let state = STATE.get_unchecked();
        let addresses = &state.addresses;
        let config = &state.config;
        let triggers = &state.triggers;

        let mut force_retire = false;

        let end_skip = config.skip_quest_complete_wait || triggers.skip_wait;
        if end_skip {
            SKIPPING = true;
        }

        if let Some(quest) = addresses.quest() {
            if end_skip && quest.complete() {
                quest.set_time_remaining(0);
            }

            if config.retire_on_death.enabled {
                match config.retire_on_death.kind {
                    DeathRetireKind::HealthDepleted => {
                        let player = addresses.own_player().unwrap_unchecked();
                        if player.health(
                            addresses.encryption1,
                            addresses.encryption2,
                            addresses.encryption3,
                        ) == 0
                        {
                            force_retire = true;
                        }
                    }
                    DeathRetireKind::Carted => {
                        if quest.carted_count() >= config.retire_on_death.carts_needed
                            && quest.remaining_carts() != 0
                        {
                            force_retire = true;
                        }
                    }
                }
            }
        }

        if force_retire || triggers.retire {
            SKIPPING = true;
            let player_info = addresses.player_info().unwrap_unchecked();
            player_info.set_retire();
        }
    }
}

pub unsafe extern "C" fn on_quest_end() {
    unsafe {
        if SKIPPING {
            let addresses = STATE.get_unchecked().addresses;
            if let Some(player_info) = addresses.player_info() {
                player_info.skip_timers();
            }
            let iframe_flags = addresses.iframe_flags();
            iframe_flags.write(0);
        }
    }
}
