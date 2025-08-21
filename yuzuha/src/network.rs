use std::ffi::CString;

use yuzuha_codegen::use_offsets;

use crate::{BASE, PtrToStringAnsi, interceptor::Interceptor, util};

#[use_offsets(INFOMANAGER_SET_APIURL, PTR_TO_STRING_ANSI)]
pub fn hook_info_manager_set_api_url(interceptor: &mut Interceptor) {
    const MANJUU_LOCAL: &str = "http://127.0.0.1:5000";
    interceptor.attach(INFOMANAGER_SET_APIURL, |ctx| {
        let url = util::read_csharp_string(ctx.registers().rcx as *const u8);
        let new_url = MANJUU_LOCAL.to_string();
        let ptr_to_string_ansi = unsafe {
            std::mem::transmute::<usize, PtrToStringAnsi>(BASE.get().unwrap() + PTR_TO_STRING_ANSI)
        };
        println!("UnityWebRequest: \"{url}\", replacing with \"{new_url}\" (manjuu redirect)");
        ctx.registers().rcx = ptr_to_string_ansi(
            CString::new(new_url.as_str())
                .unwrap()
                .to_bytes_with_nul()
                .as_ptr(),
            );
    });
}

#[use_offsets(QT5_CORE_QURL)]
pub fn hook_qt5_core_qurl(interceptor: &mut Interceptor) {
    const MANJUU_LOCAL: &str = "http://127.0.0.1:5000";

    interceptor.attach(QT5_CORE_QURL, |ctx| {
        let ptr = unsafe {
            *(ctx.registers().rdx as *const *const u8) // 解一次引用
        };
        let url = util::read_qurl_string(ptr);
        println!("Intercepted URL: {}", url);
        
        if !url.is_empty()  && url.contains("manjuu.com") {
            let mut new_url = MANJUU_LOCAL.to_string();
            println!("UnityWebRequest: \"{url}\", replacing with \"{new_url}\" (manjuu redirect)");
            url.split('/').skip(3).for_each(|s| {
                new_url.push('/');
                new_url.push_str(s);
            });
            let new_url_utf16: Vec<u16> = new_url.encode_utf16().collect(); // utf-8 -> utf-16
            if new_url_utf16.len() <= url.len() {
                unsafe {
                    let url_str = ptr.byte_offset(0x18).cast::<u16>().cast_mut();
                    *ptr.byte_offset(0x4).cast::<u32>().cast_mut() = new_url_utf16.len() as u32; // str_len
                    *ptr.byte_offset(0x8).cast::<u32>().cast_mut() = (new_url_utf16.len() + 1) as u32; // str_len + 1

                    std::ptr::write_bytes(url_str, 0, url.len());
                    std::ptr::copy_nonoverlapping(
                        new_url_utf16.as_ptr(),
                        url_str,
                        new_url_utf16.len(),
                    );
                }
            } 
        }
    });
}