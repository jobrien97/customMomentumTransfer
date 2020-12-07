
use smash::lib::{self, lua_const::*};
use smash::app::{self, lua_bind::*};


pub fn get_category(boma: &mut app::BattleObjectModuleAccessor) -> i32{
    return (boma.info >> 28) as u8 as i32;
}
extern "C"{
    #[link_name = "\u{1}_ZN3app7utility8get_kindEPKNS_26BattleObjectModuleAccessorE"]
    pub fn get_kind(module_accessor: &mut app::BattleObjectModuleAccessor) -> i32;
}
pub unsafe fn get_player_number(boma: &mut app::BattleObjectModuleAccessor) -> usize{
    app::lua_bind::WorkModule::get_int(boma, *FIGHTER_INSTANCE_WORK_ID_INT_ENTRY_ID) as usize
}

pub unsafe fn clamp(x: f32, min: f32, max: f32) -> f32 {
    return if x < min { min } else if x < max { x } else { max };
}