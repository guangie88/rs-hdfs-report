#![cfg_attr(feature = "cargo-clippy",
            allow(redundant_field_names, suspicious_arithmetic_impl))]

// currently bitflags has issue with the above two lint types
// due to the way the macro expands
bitflags! {
    pub struct SubPerm: u8 {
        const NIL = 0b0000_0000;
        const EXEC = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const READ = 0b0000_0100;
        const ALL = Self::READ.bits | Self::WRITE.bits | Self::EXEC.bits;
    }
}
