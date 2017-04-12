extern crate serde_osc;
#[macro_use]
extern crate serde_osc_macros;
use serde_osc::NumFields;

#[derive(NumFields)]
struct MyStruct {
    x: u32,
}

#[test]
fn it_works2() {
    assert!(MyStruct::num_fields() == 1);
}

