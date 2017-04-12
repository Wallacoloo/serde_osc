/// Osc 1.0 supported arguments.
/// We follow the same case conventions as in the specifications.
/// Notably, some (currently unsupported) vendor extensions are case-sensitive (e.g. F and T).
#[allow(non_camel_case_types)]
pub enum OscArg {
    /// 32-bit signed integer
    i(i32),
    /// 32-bit float
    f(f32),
    /// String; specified as null-terminated ascii.
    /// This might also represent the message address pattern (aka path)
    s(String),
    /// 'blob' (binary) data
    b(Vec<u8>),
}

