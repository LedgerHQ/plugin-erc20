#![no_std]
#![no_main]

use nanos_sdk::bindings::{
    os_lib_end
};

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);
use nanos_sdk::plugin::PluginCtx;

#[no_mangle]
extern "C" fn sample_main(arg0: u32) {

    let args: *mut u32 = arg0 as *mut u32;
    let value1 = unsafe { *args };
    let value2 = unsafe { *args.add(1) as *mut PluginCtx };
    let plugin_ctx: &mut PluginCtx = unsafe { &mut *value2 };
    
    {
        nanos_sdk::testing::debug_print(core::str::from_utf8(&plugin_ctx.name).unwrap());
        nanos_sdk::testing::debug_print("\n");
    }

    for (idx, b) in "modified_by_plugin".bytes().enumerate() {
        plugin_ctx.name[idx] = b;
    }
    
    unsafe {
        os_lib_end();
    }
}