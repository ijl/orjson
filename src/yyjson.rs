#[repr(C)]
pub struct yyjson_alc {
    pub malloc: ::std::option::Option<
        unsafe extern "C" fn(
            ctx: *mut ::std::os::raw::c_void,
            size: usize,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub realloc: ::std::option::Option<
        unsafe extern "C" fn(
            ctx: *mut ::std::os::raw::c_void,
            ptr: *mut ::std::os::raw::c_void,
            size: usize,
        ) -> *mut ::std::os::raw::c_void,
    >,
    pub free: ::std::option::Option<
        unsafe extern "C" fn(ctx: *mut ::std::os::raw::c_void, ptr: *mut ::std::os::raw::c_void),
    >,
    pub ctx: *mut ::std::os::raw::c_void,
}
extern "C" {
    pub fn yyjson_alc_pool_init(
        alc: *mut yyjson_alc,
        buf: *mut ::std::os::raw::c_void,
        size: usize,
    ) -> bool;
}
pub type yyjson_read_flag = u32;
pub const YYJSON_READ_NOFLAG: yyjson_read_flag = 0;
pub type yyjson_read_code = u32;
pub const YYJSON_READ_SUCCESS: yyjson_read_code = 0;
#[repr(C)]
pub struct yyjson_read_err {
    pub code: yyjson_read_code,
    pub msg: *const ::std::os::raw::c_char,
    pub pos: usize,
}
extern "C" {
    pub fn yyjson_read_opts(
        dat: *mut ::std::os::raw::c_char,
        len: usize,
        flg: yyjson_read_flag,
        alc: *const yyjson_alc,
        err: *mut yyjson_read_err,
    ) -> *mut yyjson_doc;
}
extern "C" {
    pub fn yyjson_doc_free(doc: *mut yyjson_doc);
}
#[repr(C)]
pub union yyjson_val_uni {
    pub u64_: u64,
    pub i64_: i64,
    pub f64_: f64,
    pub str_: *const ::std::os::raw::c_char,
    pub ptr: *mut ::std::os::raw::c_void,
    pub ofs: usize,
}
#[repr(C)]
pub struct yyjson_val {
    pub tag: u64,
    pub uni: yyjson_val_uni,
}
#[repr(C)]
pub struct yyjson_doc {
    pub root: *mut yyjson_val,
    pub alc: yyjson_alc,
    pub dat_read: usize,
    pub val_read: usize,
    pub str_pool: *mut ::std::os::raw::c_char,
}
