use std::{sync::OnceLock, thread, time::Duration};

use interceptor::Interceptor;
use windows::{
    Win32::{
        Foundation::HINSTANCE,
        System::{Console, LibraryLoader::GetModuleHandleA, SystemServices::DLL_PROCESS_ATTACH},
    },
    core::s,
};

mod interceptor;
mod network;
mod util;

type PtrToStringAnsi = extern "C" fn(chars: *const u8) -> u64;
static BASE: OnceLock<usize> = OnceLock::new();
static QT5_CORE_BASE: OnceLock<usize> = OnceLock::new();

fn on_attach() {
    // SAFETY: fuck off
    thread::sleep(Duration::from_secs(3)); //如果控制台不打印信息,代码也会正常执行
    unsafe {                               //能在客户端发送请求之前 hook 成功即可
        let _ = Console::FreeConsole();
        let _ = Console::AllocConsole();
    }
    println!("yuzuha-patch (For Blue Genshin) is initializing");

    let base = loop {
        unsafe {
            match GetModuleHandleA(s!("GameAssembly.dll")) {
                Ok(handle) => {
                    let base_addr = handle.0 as usize;
                    println!("GameAssembly.dll Addr: 0x{:X}", base_addr);
                    break base_addr;
                },
                Err(_) => thread::sleep(Duration::from_millis(200)),
            }
        }
    };
    
    let _qt5_core_base = loop {
        unsafe {
            match GetModuleHandleA(s!("Qt5Core.dll")) {
                Ok(handle) => {
                    let base_addr = handle.0 as usize;
                    println!("Qt5Core.dll Addr: 0x{:X}", base_addr);
                    break base_addr;
                },
                Err(_) => thread::sleep(Duration::from_millis(200)),
            }
        }
    };

    //thread::sleep(Duration::from_secs(5)); 不要过多使用sleep,要在客户端发送请求前hook成功
    util::disable_memory_protection();

    let _ = BASE.set(base);
    let _ = QT5_CORE_BASE.set(_qt5_core_base);

    let mut interceptor = Interceptor::new(base);
    let mut interceptor_qurl = Interceptor::new(_qt5_core_base);

    network::hook_info_manager_set_api_url(&mut interceptor);
    network::hook_qt5_core_qurl(&mut interceptor_qurl);

    std::thread::sleep(Duration::from_secs(u64::MAX));
}

#[unsafe(no_mangle)]
#[allow(non_snake_case)]
unsafe extern "system" fn DllMain(_: HINSTANCE, call_reason: u32, _: *mut ()) -> bool {
    if call_reason == DLL_PROCESS_ATTACH {
        thread::spawn(on_attach);
    }

    true
}
