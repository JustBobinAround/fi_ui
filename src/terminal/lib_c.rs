use std::ffi::c_int;

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibCErr(c_int);

impl LibCErr {
    pub fn into_result(self) -> Result<(), Self> {
        if self.0 == 0 { Ok(()) } else { Err(self) }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct LibCBool(c_int);

impl LibCErr {
    pub fn as_bool(self) -> bool {
        self.0 != 0
    }
}
