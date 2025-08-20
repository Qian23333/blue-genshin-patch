#![allow(clippy::missing_transmute_annotations)]

use std::ffi::CString;
use std::process::ExitCode;

use windows::Win32::Foundation::{CloseHandle, GetLastError, HANDLE};
use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows::Win32::System::Memory::{
    MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE, VirtualAllocEx, VirtualFreeEx,
};
use windows::Win32::System::Threading::{
    CREATE_SUSPENDED, CreateProcessA, CreateRemoteThread, PROCESS_INFORMATION, ResumeThread,
    STARTUPINFOA, WaitForSingleObject,
};
use windows::core::{PCSTR, s};

const EXECUTABLES: &[&str] = &["ZenlessZoneZero.exe", "ZenlessZoneZeroBeta.exe"];
const INJECT_DLL: &str = "yuzuha.dll";

fn main() -> ExitCode {
    let current_dir = std::env::current_dir().unwrap();

    // 1. 获取上级目录
    let parent_dir = match current_dir.parent() {
        Some(p) => p,
        None => {
            eprintln!("错误: 无法获取上级目录。");
            let _ = std::io::stdin().read_line(&mut String::new());
            return ExitCode::FAILURE;
        }
    };

    // 2. 构建 DLL 的搜索路径 (../yuzuha/)
    let dll_search_dir = parent_dir.join("yuzuha");

    // 3. 定义 DLL 的完整路径
    let dll_path = dll_search_dir.join(INJECT_DLL);
    if !dll_path.is_file() {
        eprintln!("{INJECT_DLL} not found");
        let _ = std::io::stdin().read_line(&mut String::new());
        return ExitCode::FAILURE;
    }

    for &exe_name in EXECUTABLES {
        if current_dir.join(exe_name).is_file() {
            eprintln!("Found game executable: {exe_name}");
            let exe_name = CString::new(exe_name).unwrap();
            let mut proc_info = PROCESS_INFORMATION::default();
            let startup_info = STARTUPINFOA::default();

            unsafe {
                CreateProcessA(
                    PCSTR(exe_name.as_bytes_with_nul().as_ptr()),
                    None,
                    None,
                    None,
                    false,
                    CREATE_SUSPENDED,
                    None,
                    None,
                    &startup_info,
                    &mut proc_info,
                )
                .unwrap();

                if inject_standard(proc_info.hProcess, dll_path.to_str().unwrap()) {
                    ResumeThread(proc_info.hThread);
                }

                CloseHandle(proc_info.hThread).unwrap();
                CloseHandle(proc_info.hProcess).unwrap();
            }

            return ExitCode::SUCCESS;
        }
    }

    eprintln!("can't find game executable in this directory");
    let _ = std::io::stdin().read_line(&mut String::new());

    ExitCode::FAILURE
}

fn inject_standard(h_target: HANDLE, dll_path: &str) -> bool {
    unsafe {
        let loadlib = GetProcAddress(
            GetModuleHandleA(s!("kernel32.dll")).unwrap(),
            s!("LoadLibraryA"),
        )
        .unwrap();

        let dll_path_cstr = CString::new(dll_path).unwrap();
        let dll_path_addr = VirtualAllocEx(
            h_target,
            None,
            dll_path_cstr.to_bytes_with_nul().len(),
            MEM_COMMIT | MEM_RESERVE,
            PAGE_READWRITE,
        );

        if dll_path_addr.is_null() {
            println!("VirtualAllocEx failed. Last error: {:?}", GetLastError());
            return false;
        }

        WriteProcessMemory(
            h_target,
            dll_path_addr,
            dll_path_cstr.as_ptr() as _,
            dll_path_cstr.to_bytes_with_nul().len(),
            None,
        )
        .unwrap();

        let h_thread = CreateRemoteThread(
            h_target,
            None,
            0,
            Some(std::mem::transmute(loadlib)),
            Some(dll_path_addr),
            0,
            None,
        )
        .unwrap();

        WaitForSingleObject(h_thread, 0xFFFFFFFF);

        VirtualFreeEx(h_target, dll_path_addr, 0, MEM_RELEASE).unwrap();
        CloseHandle(h_thread).unwrap();
        true
    }
}
