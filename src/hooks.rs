use crate::{
    address::{Addresses, Player},
    config::DeathRetireKind,
    plugin::{RETIRE, SKIP_WAIT, STATE},
};

use anyhow::Result;
use ilhook::x86::{CallbackOption, ClosureHookPoint, HookFlags, Registers, hook_closure_jmp_back};
use tracing::info;

#[allow(static_mut_refs)]
fn hook_quest_update<'a>(addresses: Addresses) -> Result<ClosureHookPoint<'a>> {
    let on_call = move |reg: *mut Registers| unsafe {
        if let Some(config) = STATE.as_ref().map(|s| &s.config) {
            if let Some(quest) = addresses.quest() {
                if (config.skip_quest_complete_wait || SKIP_WAIT.trigger()) && quest.complete() {
                    quest.set_remaining_time(0);
                    SKIP_WAIT.reset();
                }

                if config.retire_on_death.enabled {
                    match config.retire_on_death.kind {
                        DeathRetireKind::HealthDepleted => {
                            let player = Player::from_addr((*reg).ebx);
                            if player.current_health(&addresses) == 0 {
                                RETIRE.set();
                            }
                        }
                        DeathRetireKind::Carted => {
                            if quest.carted_count() >= config.retire_on_death.carts_needed {
                                RETIRE.set();
                            }
                        }
                    }
                }
            }

            if RETIRE.trigger() {
                let ptr = (*reg).edi as *mut u8;
                ptr.write(6);
                RETIRE.reset();
            }
        }
    };

    let hook_address = addresses.quest_update;
    let hook = unsafe {
        hook_closure_jmp_back(
            hook_address,
            on_call,
            CallbackOption::None,
            HookFlags::empty(),
        )?
    };
    info!("Hooked at {:#X}", hook_address);
    Ok(hook)
}

pub fn init<'a>(addresses: &Addresses) -> Result<Vec<ClosureHookPoint<'a>>> {
    Ok(vec![hook_quest_update(*addresses)?])
}
