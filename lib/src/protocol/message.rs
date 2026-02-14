#[derive(Debug)]
pub struct Message {
    pub header: Header,
    //pub questions: Vec<Question>,
    //pub answer: Answer,
    //pub authority: Authority,
    //pub additional: Additional,
}

#[derive(Debug)]
pub struct Header {
    pub id: Id,
    pub query_response: QueryResponse,
    pub opcode: OpCode,
    pub authoritative_answer: bool,
    pub truncation: bool,
    pub recursion_desired: bool,
    pub recursion_available: bool,
    pub reserved: bool,
    pub authentic_data: bool,    // rfc2535
    pub checking_disabled: bool, // rfc2535
    pub response_code: ResponseCode,
    pub qd_count: u16,
    pub an_count: u16,
    pub ns_count: u16,
    pub ar_count: u16,
}

pub type Id = u16;

#[derive(Debug)]
pub enum QueryResponse {
    Query,
    Response,
}

impl TryFrom<u8> for QueryResponse {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => QueryResponse::Query,
            1 => QueryResponse::Response,
            _ => return Err("Unknown value"),
        })
    }
}

#[derive(Debug)]
pub enum OpCode {
    Query,
    InverseQuery,
    Status,
    Reserved(u8),
}

impl TryFrom<u8> for OpCode {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => OpCode::Query,
            1 => OpCode::InverseQuery,
            2 => OpCode::Status,
            x @ 3..=15 => OpCode::Reserved(x),
            _ => return Err("Unknown value"),
        })
    }
}

#[derive(Debug)]
pub enum ResponseCode {
    NoErrorCondition,
    FormatError,
    ServerFailure,
    NameError,
    NotImplemented,
    Refused,
    Reserved(u8),
}
impl TryFrom<u8> for ResponseCode {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ResponseCode::NoErrorCondition,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            x @ 6..=15 => ResponseCode::Reserved(x),
            _ => return Err("Unknown value"),
        })
    }
}

pub struct Question {

}

pub struct Answer {}

pub struct Authority {}

pub struct Additional {}
