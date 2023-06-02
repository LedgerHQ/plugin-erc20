#![no_std]
#![no_main]

use nanos_sdk::bindings::{
    os_lib_end
};

nanos_sdk::set_panic!(nanos_sdk::exiting_panic);

use nanos_sdk::plugin::{
    PluginCoreParams,
    PluginCheckParams,
    PluginInitParams,
    PluginFeedParams,
    PluginFinalizeParams,
    PluginQueryUiParams,
    PluginGetUiParams,
    PluginInteractionType,
    PluginResult
};

use nanos_sdk::{
    string,
    testing
};

use nanos_sdk::starknet::{
    Call,
    TransactionInfo, 
    FieldElement
};

struct Selector {
    name: &'static str,
    value: [u8; 32]
}

struct Erc20Ctx {
    address: [u8; 32],
    method: &'static str,
    destination: [u8; 32],
    amount: [u8; 32],
    token_info_idx: Option<usize>,
}

const N_SELECTORS: usize = 2;

const METHODS: [&str; N_SELECTORS] = [
    "TRANSFER", 
    "APPROVE"
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

mod token;
use token::{TOKENS, TokenInfo};

#[no_mangle]
extern "C" fn sample_main(arg0: u32) {

    // to remove when PR https://github.com/LedgerHQ/ledger-nanos-sdk/pull/69 will be merged into SDK
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

    // to remove when PR https://github.com/LedgerHQ/ledger-nanos-sdk/pull/69 will be merged into SDK
    let tokens: [TokenInfo; 2] = [
        TokenInfo {
            address: [
                0x06, 0x8f, 0x5c, 0x6a, 0x61, 0x78, 0x07, 0x68, 0x45, 0x5d, 0xe6, 0x90, 0x77, 0xe0, 0x7e, 0x89, 
                0x78, 0x78, 0x39, 0xbf, 0x81, 0x66, 0xde, 0xcf, 0xbf, 0x92, 0xb6, 0x45, 0x20, 0x9c, 0x0f, 0xb8
            ],
            name: "Tether USDT",
            ticker: "USDT".as_bytes(),
            decimals: 6
        },
        TokenInfo {
            address: [
                0x04, 0x9d, 0x36, 0x57, 0x0d, 0x4e, 0x46, 0xf4, 0x8e, 0x99, 0x67, 0x4b, 0xd3, 0xfc, 0xc8, 0x46,
                0x44, 0xdd, 0xd6, 0xb9, 0x6f, 0x7c, 0x74, 0x1b, 0x15, 0x62, 0xb8, 0x2f, 0x9e, 0x00, 0x4d, 0xc7
            ],
            name: "Ether",
            ticker: "ETH".as_bytes(),
            decimals: 18
        }
    ];

    let args: *mut u32 = arg0 as *mut u32;
    
    let value1 = unsafe { *args as u16 };
    let operation: PluginInteractionType = value1.into();
    
    match operation {
        PluginInteractionType::Check => {
            testing::debug_print("Check plugin presence\n");
        }
        PluginInteractionType::Init => {
            testing::debug_print("Init plugin context\n");

            let value2 = unsafe { *args.add(1) as *mut PluginInitParams };

            let params: &mut PluginInitParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.core_params.plugin_internal_ctx as *mut Erc20Ctx)};
            let call: &Call = unsafe {&*(params.data_in as *const Call)};

            /*{
                let s = string::to_utf8::<64>(string::Value::ARR32(tx_info.sender_address.value));
                testing::debug_print(core::str::from_utf8(&s).unwrap());
                testing::debug_print("\n");
            }*/

            erc20_ctx.address = call.to.value; 
            for i in 0..N_SELECTORS {
                if call.selector.value == selectors[i].value {
                    erc20_ctx.method = selectors[i].name;
                }
            }
            params.core_params.plugin_result = PluginResult::Ok;
        }   
        PluginInteractionType::Feed => {
            testing::debug_print("Feed plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFeedParams };

            let params: &mut PluginFeedParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.core_params.plugin_internal_ctx as *mut Erc20Ctx)};
            let call: &Call = unsafe {&*(params.data_in[0] as *const Call)};

            erc20_ctx.destination = call.calldata[0].value;
            erc20_ctx.amount = call.calldata[1].value;

            {
                testing::debug_print("Token: 0x");         
                let mut s = string::to_utf8::<64>(string::Value::ARR32(erc20_ctx.address));
                testing::debug_print(core::str::from_utf8(&s).unwrap());
                testing::debug_print("\n");

                testing::debug_print("method: ");
                testing::debug_print(erc20_ctx.method);
                testing::debug_print("\n");

                testing::debug_print("destination: 0x");
                s = string::to_utf8::<64>(string::Value::ARR32(erc20_ctx.destination));
                testing::debug_print(core::str::from_utf8(&s).unwrap());
                testing::debug_print("\n");

                testing::debug_print("amount: ");
                let mut amount_string: [u8; 100] = [b'0'; 100];
                let mut amount_string_length: usize = 0;
                string::uint256_to_float(&erc20_ctx.amount, 18, &mut amount_string[..], &mut amount_string_length);
                testing::debug_print(core::str::from_utf8(&amount_string[..amount_string_length]).unwrap());
                testing::debug_print("\n");
            }

            params.core_params.plugin_result = PluginResult::Ok;
        
        }
        PluginInteractionType::Finalize => {
            testing::debug_print("Finalize plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginFinalizeParams };

            let params: &mut PluginFinalizeParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.core_params.plugin_internal_ctx as *mut Erc20Ctx)};

            erc20_ctx.token_info_idx = None;
            for i in 0..2 {
                if erc20_ctx.address == tokens[i].address {
                    erc20_ctx.token_info_idx = Some(i);
                }
            }
            params.num_ui_screens = 4;
            params.core_params.plugin_result = match erc20_ctx.token_info_idx {
                Some(idx) => {
                    testing::debug_print("token info found in plugin\n");
                    PluginResult::Ok
                }
                None => {
                    testing::debug_print("token info not found in plugin\n");
                    PluginResult::NeedInfo
                }
            };
        }
        PluginInteractionType::QueryUi => {
            testing::debug_print("QueryUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginQueryUiParams };

            let params: &mut PluginQueryUiParams = unsafe { &mut *value2 };

            let title = "ERC-20 OPERATION".as_bytes();
            params.title[..title.len()].copy_from_slice(title);
            params.title_len = title.len();
            params.core_params.plugin_result = PluginResult::Ok;
        }
        PluginInteractionType::GetUi => {
            testing::debug_print("GetUI plugin\n");

            let value2 = unsafe { *args.add(1) as *mut PluginGetUiParams };

            let params: &mut PluginGetUiParams = unsafe { &mut *value2 };
            let erc20_ctx: &mut Erc20Ctx = unsafe {&mut *(params.core_params.plugin_internal_ctx as *mut Erc20Ctx)};

            testing::debug_print("requested screen index: ");
            let mut s = string::to_utf8::<2>(string::Value::U8(params.ui_screen_idx as u8));
            testing::debug_print(core::str::from_utf8(&s).unwrap());
            testing::debug_print("\n");

            let idx = erc20_ctx.token_info_idx.expect("unknown token");
            let token = tokens[idx];

            match params.ui_screen_idx {
                0 => {
                    let title = "TOKEN:".as_bytes();
                    params.title[..title.len()].copy_from_slice(title);
                    params.title_len = title.len();

                    
                    let msg = token.name.as_bytes(); 
                    params.msg[..msg.len()].copy_from_slice(msg);
                    params.msg_len = msg.len();

                    params.core_params.plugin_result = PluginResult::Ok;
                }
                1 => {
                    let title = "METHOD:".as_bytes();
                    params.title[..title.len()].copy_from_slice(title);
                    params.title_len = title.len();

                    let msg = erc20_ctx.method.as_bytes();
                    params.msg[..msg.len()].copy_from_slice(msg);
                    params.msg_len = msg.len();

                    params.core_params.plugin_result = PluginResult::Ok;
                }
                2 => {
                    let title = "TO:".as_bytes();
                    params.title[..title.len()].copy_from_slice(title);
                    params.title_len = title.len();

                    let msg = string::to_utf8::<64>(string::Value::ARR32(erc20_ctx.destination));
                    params.msg[..64].copy_from_slice(&msg[..]);
                    params.msg_len = 64;

                    params.core_params.plugin_result = PluginResult::Ok;
                }
                3 => {
                    let title = "AMOUNT:".as_bytes();
                    params.title[..title.len()].copy_from_slice(title);
                    params.title_len = title.len();

                    let mut amount_string: [u8; 100] = [b'0'; 100];
                    let mut amount_string_length: usize = 0;
                    string::uint256_to_float(&erc20_ctx.amount, token.decimals, &mut amount_string[..], &mut amount_string_length);
                    
                    params.msg[..amount_string_length].copy_from_slice(&amount_string[..amount_string_length]);
                    params.msg_len = amount_string_length;

                    params.core_params.plugin_result = PluginResult::Ok;
                }
                _ => {
                    params.core_params.plugin_result = PluginResult::Err;
                }
            }
        }
        _ => {
            testing::debug_print("Not implemented\n");
        }
    }
    
    unsafe {
        os_lib_end();
    }
}