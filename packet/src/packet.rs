// length constants
pub const CMD_LENGTH: usize = 1;
pub const LEN_LENGTH: usize = 2;
pub const TOKEN_LENGTH: usize = 4;

// commands
pub const CMD_WRITE: u8 = 0x01;
pub const CMD_DELETE: u8 = 0x02;
pub const CMD_READ: u8 = 0x03;
pub const CMD_USE: u8 = 0x04;
pub const CMD_CURRENT_DB: u8 = 0x05;
pub const CMD_LIST_DB: u8 = 0x06;
pub const CMD_DETACH: u8 = 0x07;

pub const CMD_RANGE_BEGIN: u8 = 0x31;
pub const CMD_RANGE_END: u8 = 0x32;
pub const CMD_RANGE_FROM_ASC: u8 = 0x33;
pub const CMD_RANGE_FROM_ASC_EX: u8 = 0x34;
pub const CMD_RANGE_FROM_DESC: u8 = 0x35;
pub const CMD_RANGE_FROM_DESC_EX: u8 = 0x36;

// responses
pub const RESP_OK: u8 = 0x55;
pub const RESP_ERROR: u8 = 0x56;
pub const RESP_TOKEN: u8 = 0x57;
pub const RESP_TOKENS: u8 = 0x58;
pub const RESP_PAIRS: u8 = 0x59;

#[derive(Debug, PartialEq)]
pub enum Packet {
    // commands
    CmdWrite(Vec<Vec<u8>>),
    CmdRead(Vec<Vec<u8>>),
    CmdDelete(Vec<Vec<u8>>),
    CmdUse(Vec<u8>),
    CmdCurrentDB(),
    CmdListDb(),
    CmdDetach(Vec<u8>),

    // command-ranges
    CmdRangeBegin(u16),
    CmdRangeEnd(u16),
    CmdRangeFromAsc(u16, Vec<u8>),
    CmdRangeFromAscEx(u16, Vec<u8>),
    CmdRangeFromDesc(u16, Vec<u8>),
    CmdRangeFromDescEx(u16, Vec<u8>),

    // responses
    RespOk(String),
    RespError(String),
    RespToken(Vec<u8>),
    RespTokens(Vec<Vec<u8>>),
    RespPairs(Vec<Vec<u8>>),
}
