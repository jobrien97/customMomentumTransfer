use smash::app::{self, lua_bind::*, sv_kinetic_energy, sv_animcmd, sv_system};
use smash::lib::{lua_const::*, L2CValue, L2CAgent};
use smash::lua2cpp::L2CFighterCommon;
use smash::{phx::*, hash40};
use crate::utils::*;


//Turn dash (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_TurnDash_Sub)]
pub unsafe fn status_turndash_sub_hook(fighter: &mut L2CFighterCommon) -> L2CValue {
  let ret = original!()(fighter);
  let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
  let mut f_agent = fighter.agent;

  f_agent.clear_lua_stack();
  f_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_MOTION as u64));
  f_agent.push_lua_stack(&mut L2CValue::new_num(0.0));
  sv_kinetic_energy::set_speed(fighter.lua_state_agent);
  f_agent.clear_lua_stack();

  ret
}

//Jump (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_status_Jump_sub)]
pub unsafe fn status_jump_sub_hook(fighter: &mut L2CFighterCommon, param_2: L2CValue, param_3: L2CValue) -> L2CValue {
    let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
    let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
    let fighter_kind = app::utility::get_kind(boma);
    if fighter_kind != *FIGHTER_KIND_NANA {
        l2c_agent.clear_lua_stack();
        l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
        l2c_agent.push_lua_stack(&mut L2CValue::new_num(calc_melee_momentum(boma, false)));
        sv_kinetic_energy::set_speed(fighter.lua_state_agent);
        l2c_agent.clear_lua_stack();
    }
    original!()(fighter, param_2, param_3)
}


//Aerials (runs once at the beginning of the status)
#[skyline::hook(replace = smash::lua2cpp::L2CFighterCommon_sub_attack_air_common)]
pub unsafe fn status_attack_air_hook(fighter: &mut L2CFighterCommon, param_1: L2CValue){
    let lua_state = fighter.lua_state_agent;
    let boma = smash::app::sv_system::battle_object_module_accessor(lua_state);
    let fighter_kind = app::utility::get_kind(boma);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("run_speed_max"), 0) * jump_speed_ratio[get_player_number(boma)];

    if ![*FIGHTER_KIND_NESS, *FIGHTER_KIND_MEWTWO, *FIGHTER_KIND_LUCAS, *FIGHTER_KIND_RYU, *FIGHTER_KIND_KEN, *FIGHTER_KIND_NANA, *FIGHTER_KIND_POPO, *FIGHTER_KIND_SIMON, *FIGHTER_KIND_RICHTER, *FIGHTER_KIND_DOLLY].contains(&fighter_kind) {
        let boma = app::sv_system::battle_object_module_accessor(fighter.lua_state_agent);
        let mut l2c_agent = L2CAgent::new(fighter.lua_state_agent);
        let is_speed_backward = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN) * PostureModule::lr(boma) < 0.0;
        let prev_status_check = [*FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_JUMP_SQUAT].contains(&StatusModule::prev_status_kind(boma, 0));
        let mut new_speed = clamp(curr_momentum[get_player_number(boma)], -jump_speed_x_max, jump_speed_x_max);

            /*      Shorthop aerial macro and "bair stick flick" fix     */
        if WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_FRAME_IN_AIR) <= 1 &&
            StatusModule::prev_status_kind(boma, 1) == *FIGHTER_STATUS_KIND_JUMP_SQUAT && !is_speed_backward && ![*FIGHTER_STATUS_KIND_PASS, *FIGHTER_STATUS_KIND_FALL].contains(&StatusModule::prev_status_kind(boma, 0)) { //if you used the shorthop aerial macro or just dropped through a plat
            new_speed = calc_melee_momentum(boma, true);
        }

        if prev_status_check {
            l2c_agent.clear_lua_stack();
            l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
            l2c_agent.push_lua_stack(&mut L2CValue::new_num(new_speed));
            sv_kinetic_energy::set_speed(fighter.lua_state_agent);
            l2c_agent.clear_lua_stack();
        }
    }
    original!()(fighter, param_1)
}


//called in moveset_edits in sys_line_system_control_fighter.rs
pub static mut jump_speed_ratio: [f32;8] = [0.0;8];
pub static mut curr_momentum: [f32;8] = [0.0;8];
pub static mut curr_momentum_specials: [f32;8] = [0.0;8];
pub static mut js_vel: [f32;8] = [0.0;8];
pub static mut rar_leniency: [f32;8] = [0.0;8];
pub static mut is_footstool: bool = false;
pub unsafe fn momentum_transfer_helper(lua_state: u64, l2c_agent: &mut L2CAgent, boma: &mut app::BattleObjectModuleAccessor, status_kind: i32, situation_kind: i32, curr_frame: f32, fighter_kind: i32) {
    let dash_speed = WorkModule::get_param_float(boma, hash40("dash_speed"), 0);
    let ground_brake = WorkModule::get_param_float(boma, hash40("ground_brake"), 0);

    if status_kind == *FIGHTER_STATUS_KIND_ENTRY {
        jump_speed_ratio[get_player_number(boma)] = (WorkModule::get_param_float(boma, hash40("jump_speed_x_max"), 0) / WorkModule::get_param_float(boma, hash40("run_speed_max"), 0));
    }

    let dash_speed: f32 = WorkModule::get_param_float(boma, hash40("dash_speed"), 0);
    let pivot_boost: smash::phx::Vector3f = smash::phx::Vector3f {x: (dash_speed * 0.75), y: 0.0, z: 0.0};

    if status_kind == *FIGHTER_STATUS_KIND_TURN_DASH {
        StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_DASH, true);
    }

    if status_kind == *FIGHTER_STATUS_KIND_TURN_DASH && curr_frame <= 1.0 && [*FIGHTER_STATUS_KIND_TURN_DASH, *FIGHTER_STATUS_KIND_DASH].contains(&StatusModule::prev_status_kind(boma, 0)) && ![*FIGHTER_STATUS_KIND_WAIT, *FIGHTER_STATUS_KIND_TURN].contains(&StatusModule::prev_status_kind(boma, 1)) {
        if ControlModule::get_stick_x(boma) == 0.0 {
            PostureModule::reverse_lr(boma);
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_TURN, true);
            KineticModule::clear_speed_all(boma);
            KineticModule::add_speed(boma, &pivot_boost);
        }
    }

    if status_kind == *FIGHTER_STATUS_KIND_DASH && curr_frame <= 1.0 && [*FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 0)) && [*FIGHTER_STATUS_KIND_TURN_DASH, *FIGHTER_STATUS_KIND_DASH].contains(&StatusModule::prev_status_kind(boma, 1)) && ![*FIGHTER_STATUS_KIND_WAIT, *FIGHTER_STATUS_KIND_TURN].contains(&StatusModule::prev_status_kind(boma, 2)) {
        if ControlModule::get_stick_x(boma) == 0.0 {
            PostureModule::reverse_lr(boma);
            StatusModule::change_status_request_from_script(boma, *FIGHTER_STATUS_KIND_TURN, true);
            KineticModule::clear_speed_all(boma);
            KineticModule::add_speed(boma, &pivot_boost);
        }
    }

    if [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE].contains(&status_kind) {
        rar_leniency[get_player_number(boma)] = clamp(0.8*(MotionModule::end_frame(boma) - MotionModule::frame(boma)*2.0 + 6.0)/MotionModule::end_frame(boma), 0.1, 0.8); // You have a limited amount of time to get full RAR momentum from turn brake or run brake, with a 3F leniency
    }

    if status_kind == *FIGHTER_STATUS_KIND_JUMP_SQUAT && curr_frame <= 1.0 {
        js_vel[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }

    if MotionModule::motion_kind(boma) == hash40("step_jump") {
        is_footstool = true;
    }

    if [*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP, *FIGHTER_STATUS_KIND_PASS].contains(&status_kind) || situation_kind == *SITUATION_KIND_AIR {
        curr_momentum[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }

    if [*FIGHTER_STATUS_KIND_JUMP_SQUAT, *FIGHTER_STATUS_KIND_JUMP].contains(&status_kind) {
        curr_momentum_specials[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
    }


            /*      ADDITIONAL MOVES THAT SHOULD CONSERVE MOMENTUM       */
    let mut should_conserve_momentum = false;

    if situation_kind == *SITUATION_KIND_AIR && curr_frame <= 1.0 {

        if [*FIGHTER_KIND_CAPTAIN, *FIGHTER_KIND_MARIO, *FIGHTER_KIND_LUIGI, *FIGHTER_KIND_PIKACHU, *FIGHTER_KIND_PICHU]
            .contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N { //put any fighter here whose neutral special should conserve momentum
                if StatusModule::prev_status_kind(boma, 0) == *FIGHTER_STATUS_KIND_JUMP {
                    should_conserve_momentum = true; //falcon punch, mario.luigi fireball
                }
                else {
                    curr_momentum_specials[get_player_number(boma)] = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);
                    should_conserve_momentum = true; //falcon punch, mario.luigi fireball
                }
        }

        if should_conserve_momentum && KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN).abs() > 0.1 {
            l2c_agent.clear_lua_stack();
            l2c_agent.push_lua_stack(&mut L2CValue::new_int(*FIGHTER_KINETIC_ENERGY_ID_CONTROL as u64));
            l2c_agent.push_lua_stack(&mut L2CValue::new_num(curr_momentum_specials[get_player_number(boma)]));
            sv_kinetic_energy::set_speed(lua_state);
            l2c_agent.clear_lua_stack();
        }

    }

}

                /*      SPACIE LASER MOMENTUM   */

//called in double_jump_cancels.rs in the change_kinetic hook
pub unsafe fn change_kinetic_momentum_related(boma: &mut app::BattleObjectModuleAccessor, kinetic_type: i32) -> Option<i32> { //spacie laser momentum conservation
    let status_kind = StatusModule::status_kind(boma);
    let fighter_kind = get_kind(boma);
    let mut should_change_kinetic = false;
    if [*FIGHTER_KIND_FALCO, *FIGHTER_KIND_FOX].contains(&fighter_kind) && status_kind == *FIGHTER_STATUS_KIND_SPECIAL_N {
        should_change_kinetic = true;
    }
    if should_change_kinetic {
        return Some(-1);
    }
    None
}


unsafe fn calc_melee_momentum(boma: &mut app::BattleObjectModuleAccessor, is_aerial_attack: bool) -> f32 {

                                /*          Normal Momentum         */

    let jump_speed_x = WorkModule::get_param_float(boma, hash40("jump_speed_x"), 0);
    let jump_speed_x_mul = WorkModule::get_param_float(boma, hash40("jump_speed_x_mul"), 0);
    let stick_x = ControlModule::get_stick_x(boma);
    let jump_speed_x_max = WorkModule::get_param_float(boma, hash40("run_speed_max"), 0) * jump_speed_ratio[get_player_number(boma)];
    let x_vel = KineticModule::get_sum_speed_x(boma, *KINETIC_ENERGY_RESERVE_ATTRIBUTE_MAIN);

    let mut calcJumpSpeed = 0.0;

    if StatusModule::prev_status_kind(boma, 0) != *FIGHTER_STATUS_KIND_JUMP_SQUAT && StatusModule::prev_status_kind(boma, 1) != *FIGHTER_STATUS_KIND_JUMP_SQUAT {
        if is_footstool {
            calcJumpSpeed = 0.0;
        }
        else if [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_WALK].contains(&StatusModule::prev_status_kind(boma, 1)) || ([*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_DASH, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_END, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_TURN, *FIGHTER_STATUS_KIND_SPECIAL_LW, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_END].contains(&StatusModule::prev_status_kind(boma, 0)) && [*FIGHTER_KIND_SONIC].contains(&get_kind(boma))) {
            calcJumpSpeed = ((jump_speed_x * x_vel.signum()) + (jump_speed_x_mul * x_vel));
        }
        else {
            calcJumpSpeed = ((jump_speed_x * stick_x) + (jump_speed_x_mul * x_vel * 0.55));
        }
    }
    else {
        if [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack || [*FIGHTER_STATUS_KIND_DASH, *FIGHTER_STATUS_KIND_RUN, *FIGHTER_STATUS_KIND_TURN_DASH].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack {
            if js_vel[get_player_number(boma)].abs() >= 0.9 {
                if stick_x * PostureModule::lr(boma) < 0.0 && !is_aerial_attack {
                    calcJumpSpeed = ((jump_speed_x * stick_x * 0.1) + (jump_speed_x_mul * x_vel));
                }
                else {
                    calcJumpSpeed = ((jump_speed_x * js_vel[get_player_number(boma)].signum()) + (jump_speed_x_mul * js_vel[get_player_number(boma)]));
                }
            }
            else {
                calcJumpSpeed = (jump_speed_x * stick_x);
            }
        }
        else if [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack || [*FIGHTER_STATUS_KIND_TURN_RUN, *FIGHTER_STATUS_KIND_TURN_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack {
            if stick_x * PostureModule::lr(boma) < 0.0 && !is_aerial_attack {
                calcJumpSpeed = ((jump_speed_x * js_vel[get_player_number(boma)].signum() * rar_leniency[get_player_number(boma)]) + (jump_speed_x_mul * js_vel[get_player_number(boma)]));
            }
            else {
                calcJumpSpeed = ((jump_speed_x * stick_x) + (jump_speed_x_mul * js_vel[get_player_number(boma)] * 0.1));
            }
        }

        else if [*FIGHTER_STATUS_KIND_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack || [*FIGHTER_STATUS_KIND_RUN_BRAKE].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack {
            if stick_x * PostureModule::lr(boma) < 0.0 && !is_aerial_attack {
                calcJumpSpeed = ((jump_speed_x * stick_x * 0.1) + (jump_speed_x_mul * x_vel));
            }
            else {
                calcJumpSpeed = ((jump_speed_x * js_vel[get_player_number(boma)].signum() * rar_leniency[get_player_number(boma)]) + (jump_speed_x_mul * js_vel[get_player_number(boma)]));
            }
        }

        else if [*FIGHTER_STATUS_KIND_WALK, *FIGHTER_STATUS_KIND_WALK_BRAKE].contains(&StatusModule::prev_status_kind(boma, 1)) && !is_aerial_attack || [*FIGHTER_STATUS_KIND_WALK, *FIGHTER_STATUS_KIND_WALK_BRAKE].contains(&StatusModule::prev_status_kind(boma, 2)) && is_aerial_attack {
            if stick_x * PostureModule::lr(boma) < 0.0 && !is_aerial_attack {
                calcJumpSpeed = ((jump_speed_x * stick_x * 0.1) + (jump_speed_x_mul * x_vel * 0.55));
            }
            else {
                calcJumpSpeed = ((jump_speed_x * js_vel[get_player_number(boma)].signum() * stick_x.signum() * stick_x) + (jump_speed_x_mul * js_vel[get_player_number(boma)] * 0.55));
            }
        }
        else {
            if js_vel[get_player_number(boma)].abs() >= 0.9 {
                if stick_x * PostureModule::lr(boma) < 0.0 && !is_aerial_attack {
                    calcJumpSpeed = ((jump_speed_x * stick_x * 0.1) + (jump_speed_x_mul * x_vel));
                }
                else {
                    calcJumpSpeed = ((jump_speed_x * js_vel[get_player_number(boma)].signum()) + (jump_speed_x_mul * js_vel[get_player_number(boma)]));
                }
            }
            else {
                calcJumpSpeed = (jump_speed_x * stick_x);
            }
        }
    }

    is_footstool = false;


    // Exceptions for special moves that should retain momentum
    if    ([*FIGHTER_IKE_STATUS_KIND_SPECIAL_S_DASH].contains(&StatusModule::prev_status_kind(boma, 1)) && [*FIGHTER_KIND_IKE].contains(&get_kind(boma)))
       || ([*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_PIT_STATUS_KIND_SPECIAL_S_END].contains(&StatusModule::prev_status_kind(boma, 1)) && [*FIGHTER_KIND_PIT].contains(&get_kind(boma)))
       || ([*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_PZENIGAME_STATUS_KIND_SPECIAL_S_LOOP, *FIGHTER_PZENIGAME_STATUS_KIND_SPECIAL_S_END].contains(&StatusModule::prev_status_kind(boma, 1)) && [*FIGHTER_KIND_PZENIGAME].contains(&get_kind(boma)))
       || ([*FIGHTER_STATUS_KIND_SPECIAL_S, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_DASH, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_END, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_S_TURN, *FIGHTER_STATUS_KIND_SPECIAL_LW, *FIGHTER_SONIC_STATUS_KIND_SPECIAL_LW_END].contains(&StatusModule::prev_status_kind(boma, 1)) && [*FIGHTER_KIND_SONIC].contains(&get_kind(boma))){
        calcJumpSpeed = ((jump_speed_x * x_vel.signum() * 0.5) + (jump_speed_x_mul * x_vel));
    }

    let mut jumpSpeedClamped = clamp(calcJumpSpeed, -jump_speed_x_max, jump_speed_x_max);  //melee jump speed calculation... courtesey of Brawltendo
    jumpSpeedClamped
}
