use crate::x86::X86;

pub mod ddraw;
pub mod gdi32;
pub mod kernel32;
pub mod user32;

pub unsafe fn smuggle<T: ?Sized>(x: &T) -> &'static T {
    std::mem::transmute(x)
}

// winapi is stdcall, which means args are right to left and callee-cleaned.
// The caller of winapi functions is responsible for pushing/popping the
// return address, because some callers actually 'jmp' directly.
//
// This macro generates shim wrappers of functions, taking their
// input args off the stack and forwarding their return values via eax.
#[macro_export]
macro_rules! winapi_shims {
    ($(fn $name:ident($($param:ident: $type:ident),* $(,)?);)*) => {
        pub mod shims {
            use crate::x86::X86;

            $(#[allow(non_snake_case)]
            pub fn $name(x86: &mut X86) {
                $(let $param: $type = x86.pop();)*
                x86.regs.eax = super::$name(x86, $($param),*);
            })*
        }
    }
}

// This macro runs the above and also generates a resolve()
// function that maps symbol names to shims.
#[macro_export]
macro_rules! winapi {
    ($(fn $name:ident($($param:ident: $type:ident),* $(,)?);)*) => {
        use crate::winapi_shims;
        winapi_shims!($(fn $name($($param: $type),*);)*);

        pub fn resolve(name: &str) -> Option<fn(&mut X86)> {
            Some(match name {
                $(stringify!($name) => shims::$name,)*
                _ => return None,
            })
        }
    }
}

pub fn resolve(dll: &str, sym: &str) -> Option<fn(&mut X86)> {
    match dll {
        "ddraw.dll" => ddraw::resolve(sym),
        "gdi32.dll" => gdi32::resolve(sym),
        "kernel32.dll" => kernel32::resolve(sym),
        "user32.dll" => user32::resolve(sym),
        _ => None,
    }
    // .or_else(|| {
    //     log::warn!("unresolved symbol {}:{}", dll, sym);
    //     None
    // })
}

pub struct State {
    pub ddraw: ddraw::State,
    pub gdi32: gdi32::State,
    pub kernel32: kernel32::State,
    pub user32: user32::State,
}
impl State {
    pub fn new() -> Self {
        State {
            ddraw: ddraw::State::new_empty(),
            gdi32: gdi32::State::new(),
            kernel32: kernel32::State::new(),
            user32: user32::State::new(),
        }
    }
}
