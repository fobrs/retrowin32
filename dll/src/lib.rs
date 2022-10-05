#![no_std]
#![no_main]

#[panic_handler]
#[allow(deref_nullptr)]
fn panic(_panic: &core::panic::PanicInfo<'_>) -> ! {
    unsafe {
        *(0 as *mut u8) = 0;
    }
    loop {}
}

// #[link(name="retrowin32")]
// extern "system" {
//     fn syscall(_: u32) -> u32;
// }

fn a1() -> usize {
    7
}

fn a2() -> usize {
    //unsafe { syscall(9);}
    9
}
fn a3() -> usize {
    13
}

#[repr(C)]
struct MyVtable {
    a1: fn() -> usize,
    a2: fn() -> usize,
    a3: fn() -> usize,
}

const VTAB: MyVtable = MyVtable { a1, a2, a3 };

#[no_mangle]
extern "system" fn _DllMainCRTStartup(_: *const u8, _: u32, _: *const u8) -> u32 {
    1
}

#[no_mangle]
extern "system" fn vtab() -> *const MyVtable {
    &VTAB as *const MyVtable
}
