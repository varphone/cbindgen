use std::ffi::CStr;

pub const FOO: i32 = 10;
mod aka {
    pub(crate) const AKA: u32 = 123;

    pub(crate) const AKA_FROM_SELF: u32 = self::AKA;

    pub(crate) mod nested {
        pub(crate) const AKA: u32 = 456;
        pub(crate) const AKA_FROM_SUPER: u32 = super::AKA;
    }
}

pub(crate) const PRIVATE_FORWARD_SOURCE: u32 = 321;

pub const AKA_FROM_MODULE: u32 = aka::AKA;
pub const AKA_FROM_SELF_PATH: u32 = aka::AKA_FROM_SELF;
pub const AKA_FROM_CRATE_PATH: u32 = crate::aka::AKA;
pub const AKA_FROM_NESTED_MODULE: u32 = crate::aka::nested::AKA;
pub const AKA_FROM_SUPER_PATH: u32 = crate::aka::nested::AKA_FROM_SUPER;
pub const AKA_FROM_PRIVATE: u32 = PRIVATE_FORWARD_SOURCE;
pub const BAR: &'static str = "hello world";
pub const CSTR: &CStr = c"hello world";
pub const DELIMITER: char = ':';
pub const LEFTCURLY: char = '{';
pub const QUOTE: char = '\'';
pub const TAB: char = '\t';
pub const NEWLINE: char = '\n';
pub const HEART: char = '❤';
pub const EQUID: char = '𐂃';
pub const ZOM: f32 = 3.14;

pub(crate) const DONT_EXPORT_CRATE: i32 = 20;
const DONT_EXPORT_PRIV: i32 = 30;

/// A single-line doc comment.
pub const POS_ONE: i8 = 1;
/// A
/// multi-line
/// doc
/// comment.
pub const NEG_ONE: i8 = -1;

// Some doc for shifting //
pub const SHIFT: i64 = 3;
pub const XBOOL: i64 = 1;
pub const XFALSE: i64 = (0 << SHIFT) | XBOOL;
pub const XTRUE: i64 = 1 << (SHIFT | XBOOL);

pub const CAST: u8 = 'A' as u8;
pub const DOUBLE_CAST: u32 = 1 as f32 as u32;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}

#[no_mangle]
pub extern "C" fn root(x: Foo) {}
