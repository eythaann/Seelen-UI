// https://stackoverflow.com/questions/47300622/meaning-of-flags-in-process-extended-basic-information-struct
#[allow(dead_code)]
pub enum ProcessInformationFlag {
    IsProtectedProcess = 0x1,
    IsWow64Process = 0x2,
    IsProcessInJob = 0x4,
    IsCrossSessionCreate = 0x8,
    IsFrozen = 0x10,
    IsBackground = 0x20,
    IsStronglyNamed = 0x40,
    IsSecureProcess = 0x80,
    IsSubsystemProcess = 0x100,
}
