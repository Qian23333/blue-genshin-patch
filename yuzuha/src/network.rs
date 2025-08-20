use std::ffi::CString;

use yuzuha_codegen::use_offsets;

use crate::{BASE, PtrToStringAnsi, interceptor::Interceptor, util};

#[use_offsets(MAKE_INITIAL_URL, PTR_TO_STRING_ANSI)]
pub fn hook_make_initial_url(interceptor: &mut Interceptor) {
    const MANJUU_LOCAL: &str = "http://127.0.0.1:5000";

    interceptor.attach(MAKE_INITIAL_URL, |ctx| {
        let url = util::read_csharp_string(ctx.registers().rcx as *const u8);
        println!("Intercepted URL: {}", url);

        // 仅替换 manjuu.com 到本地的逻辑（保留原路径）
        if url.contains("manjuu.com") && !url.contains("project-3") && !url.contains("launcher") {
            let mut new_url = MANJUU_LOCAL.to_string();
            url.split('/').skip(3).for_each(|s| {
                new_url.push('/');
                new_url.push_str(s);
            });

            println!("UnityWebRequest: \"{url}\", replacing with \"{new_url}\" (manjuu redirect)");
            let ptr_to_string_ansi = unsafe {
                std::mem::transmute::<usize, PtrToStringAnsi>(BASE.get().unwrap() + PTR_TO_STRING_ANSI)
            };

            ctx.registers().rcx = ptr_to_string_ansi(
                CString::new(new_url.as_str())
                    .unwrap()
                    .to_bytes_with_nul()
                    .as_ptr(),
            );
        }
    });
}