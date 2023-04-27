#![no_std]
#![no_main]

use nanos_sdk::bindings::{
    os_lib_end
};

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

use nanos_sdk::plugin::{
    PluginInitParams,
    PluginFeedParams,
    PluginInteractionType
};

use nanos_sdk::debug;

struct Erc20Ctx {
    address: [u8; 32],
    decimals: u8,
    ticker: [u8; 4],
    destination_address: [u8; 32],
    amount: [u8; 32]
}

#[no_mangle]
extern "C" fn sample_main(arg0: u32) {

    let args: *mut u32 = arg0 as *mut u32;
    
    let value1 = unsafe { *args as u16 };
    let operation: PluginInteractionType = value1.into();
    
    match operation {
        PluginInteractionType::Check => {
            debug::print("Check plugin presence\n");
        }
        PluginInteractionType::Init => {
            debug::print("Init plugin context\n");

            let value2 = unsafe { *args.add(1) as *mut PluginInitParams };

            let ctx: &mut PluginInitParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(ctx.plugin_internal_ctx as *mut Erc20Ctx)};
        
            erc20_ctx.address = [
                0x05,0x3c,0x91,0x25,0x3b,0xc9,0x68,0x2c,0x04,0x92,0x9c,0xa0,0x2e,0xd0,0x0b,0x3e,
                0x42,0x3f,0x67,0x10,0xd2,0xee,0x7e,0x0d,0x5e,0xbb,0x06,0xf3,0xec,0xf3,0x68,0xa8
            ];

            let s = debug::to_hex_string::<64>(debug::Value::ARR32(erc20_ctx.address));
            debug::print(core::str::from_utf8(&s).unwrap());
            debug::print("\n");

        }
        PluginInteractionType::Feed => {
            debug::print("Feed plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFeedParams };

            let ctx: &mut PluginFeedParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(ctx.plugin_internal_ctx as *mut Erc20Ctx)};

            let s = debug::to_hex_string::<64>(debug::Value::ARR32(erc20_ctx.address));
            debug::print(core::str::from_utf8(&s).unwrap());
            debug::print("\n");
        
        }
        _ => {
            nanos_sdk::debug::print("Not implemented\n");
        }
    }
    
    unsafe {
        os_lib_end();
    }
}