#![feature(proc_macro_hygiene)]
#![allow(non_snake_case)]
#![allow(unused_imports)]
#![allow(non_upper_case_globals)]

use skyline::nro::{self, NroInfo};
use skyline::{hook, install_hook};
use smash::app::{self, lua_bind::*, sv_system};
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use utils::*;
mod momentum_transfer;
mod utils;


fn nro_main(nro: &NroInfo) {
    match nro.name {
        "common" => {
            println!("[Momentum Transfer Plugin] Installing hooks...");
            skyline::install_hooks!(
                momentum_transfer::status_jump_sub_hook,
                momentum_transfer::status_attack_air_hook,
                momentum_transfer::status_turndash_sub_hook
            );
        }
        _ => (),
    }
}

//#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sys_line_system_control_fighter)]
pub fn sys_line_system_control_fighter_hook(fighter: &mut L2CFighterCommon) /*-> L2CValue*/ {
    unsafe {
        let boma = sv_system::battle_object_module_accessor(fighter.lua_state_agent);
        let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
        let lua_state = fighter.lua_state_agent;
        let battle_object_category = get_category(boma);


        if battle_object_category == *BATTLE_OBJECT_CATEGORY_FIGHTER {
            let status_kind = StatusModule::status_kind(boma);
            let situation_kind = StatusModule::situation_kind(boma);
            let curr_frame = MotionModule::frame(boma);
            let fighter_kind = get_kind(boma);
            momentum_transfer::momentum_transfer_helper(lua_state, &mut l2c_agent, boma, status_kind, situation_kind, curr_frame, fighter_kind);
        }
    }
}


#[skyline::main(name = "MomentumTransfer")]
pub fn main() {
    nro::add_hook(nro_main).unwrap();
    unsafe{ acmd::add_acmd_load_hook(sys_line_system_control_fighter_hook, |_, _| false); }
}
