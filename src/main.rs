#![no_std]
#![no_main]

use nanos_sdk::bindings::{
    os_lib_end
};

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

use nanos_sdk::plugin::{
    PluginInitParams,
    PluginFeedParams,
    PluginFinalizeParams,
    PluginInteractionType,
    value_to_decimal_string,
};

use nanos_sdk::debug;

struct Selector {
    name: &'static str,
    value: [u8; 32]
}

struct Erc20Ctx {
    address: [u8; 32],
    decimals: u8,
    ticker: [u8; 4],
    method: &'static str,
    destination: [u8; 32],
    amount: [u8; 32]
}

const N_SELECTORS: usize = 2;

const METHODS: [&str; N_SELECTORS] = [
    "transfer", 
    "approve"
];
const SN_KECCAK: [[u8; 32]; N_SELECTORS] = [
    [
        0x00, 0x83, 0xaf, 0xd3, 0xf4, 0xca, 0xed, 0xc6, 0xee, 0xbf, 0x44, 0x24, 0x6f, 0xe5, 0x4e, 0x38, 
        0xc9, 0x5e, 0x31, 0x79, 0xa5, 0xec, 0x9e, 0xa8, 0x17, 0x40, 0xec, 0xa5, 0xb4, 0x82, 0xd1, 0x2e
    ],
    [
        0x02, 0x19, 0x20, 0x9e, 0x08, 0x32, 0x75, 0x17, 0x17, 0x74, 0xda, 0xb1, 0xdf, 0x80, 0x98, 0x2e,
        0x9d, 0xf2, 0x09, 0x65, 0x16, 0xf0, 0x63, 0x19, 0xc5, 0xc6, 0xd7, 0x1a, 0xe0, 0xa8, 0x48, 0x0c
    ]
];

const SELECTORS: [Selector; N_SELECTORS] = [
    Selector {
        name: "transfer",
        value: [
            0x00, 0x83, 0xaf, 0xd3, 0xf4, 0xca, 0xed, 0xc6, 0xee, 0xbf, 0x44, 0x24, 0x6f, 0xe5, 0x4e, 0x38, 
            0xc9, 0x5e, 0x31, 0x79, 0xa5, 0xec, 0x9e, 0xa8, 0x17, 0x40, 0xec, 0xa5, 0xb4, 0x82, 0xd1, 0x2e
        ]
    },
    Selector {
        name: "approve",
        value: [
            0x02, 0x19, 0x20, 0x9e, 0x08, 0x32, 0x75, 0x17, 0x17, 0x74, 0xda, 0xb1, 0xdf, 0x80, 0x98, 0x2e,
            0x9d, 0xf2, 0x09, 0x65, 0x16, 0xf0, 0x63, 0x19, 0xc5, 0xc6, 0xd7, 0x1a, 0xe0, 0xa8, 0x48, 0x0c
        ]
    }
];

mod context;
use context::{Transaction};

mod token;

#[no_mangle]
extern "C" fn sample_main(arg0: u32) {

    let selectors: [Selector; 2] = [
        Selector {
            name: METHODS[0],
            value: SN_KECCAK[0]
        },
        Selector {
            name: METHODS[1],
            value: SN_KECCAK[1]
        }
    ];

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

            let params: &mut PluginInitParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.plugin_internal_ctx as *mut Erc20Ctx)};
            let tx_info: &Transaction = unsafe {&*(params.app_data as *const Transaction)};

            {
                let s = debug::to_hex_string::<64>(debug::Value::ARR32(tx_info.sender_address.value));
                debug::print(core::str::from_utf8(&s).unwrap());
                debug::print("\n");
            }

            erc20_ctx.address = tx_info.calldata_v1.calls[0].to.value;
            for i in 0..N_SELECTORS {
                if tx_info.calldata_v1.calls[0].selector.value == selectors[i].value {
                    erc20_ctx.method = selectors[i].name;
                }
            }
        }   
        PluginInteractionType::Feed => {
            debug::print("Feed plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFeedParams };

            let params: &mut PluginFeedParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.plugin_internal_ctx as *mut Erc20Ctx)};
            let tx_info: &Transaction = unsafe {&*(params.app_data as *const Transaction)};

            erc20_ctx.destination = tx_info.calldata_v1.calls[0].call_data[0].value;
            erc20_ctx.amount = tx_info.calldata_v1.calls[0].call_data[1].value;

            {
                debug::print("Token: 0x");         
                let mut s = debug::to_hex_string::<64>(debug::Value::ARR32(erc20_ctx.address));
                debug::print(core::str::from_utf8(&s).unwrap());
                debug::print("\n");

                debug::print("method: ");
                debug::print(erc20_ctx.method);
                debug::print("\n");

                debug::print("destination: 0x");
                s = debug::to_hex_string::<64>(debug::Value::ARR32(erc20_ctx.destination));
                debug::print(core::str::from_utf8(&s).unwrap());
                debug::print("\n");

                debug::print("amount: ");
                let mut amount_string: [u8; 100] = [b'0'; 100];
                let mut amount_string_length: usize = 0;
                value_to_decimal_string(&erc20_ctx.amount, 18, &mut amount_string[..], &mut amount_string_length);
                debug::print(core::str::from_utf8(&amount_string[..amount_string_length]).unwrap());
                debug::print("\n");
            }
        
        }
        PluginInteractionType::Finalize => {
            debug::print("Finalize plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFinalizeParams };

            let params: &mut PluginFinalizeParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.plugin_internal_ctx as *mut Erc20Ctx)};
            let tx_info: &Transaction = unsafe {&*(params.app_data as *const Transaction)};

        }
        _ => {
            nanos_sdk::debug::print("Not implemented\n");
        }
    }
    
    unsafe {
        os_lib_end();
    }
}