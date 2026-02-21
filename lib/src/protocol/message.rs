/*
MIT License

Copyright (c) 2026 Vincent Hiribarren

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
*/

macro_rules! int_enum_with_catchall_u16 {
    (
        $(#[$meta:meta])*
        $vis:vis enum $name:ident {
            $(
                $variant:ident = $value:expr
            ),+ $(,)?
        }
        catch_all = $catchall:ident
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        $vis enum $name {
            $(
                $variant,
            )+
            $catchall(u16),
        }

        impl From<u16> for $name {
            fn from(value: u16) -> Self {
                match value {
                    $(
                        $value => $name::$variant,
                    )+
                    v => $name::$catchall(v),
                }
            }
        }

        impl From<$name> for u16 {
            fn from(value: $name) -> Self {
                match value {
                    $(
                        $name::$variant => $value,
                    )+
                    $name::$catchall(v) => v,
                }
            }
        }
    };
}

int_enum_with_catchall_u16! {
    pub enum Class {
        Internet = 1,
        CSNet = 2,
        CHAOS = 3,
        Hesiod = 4,
    }
    catch_all = Other
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QClass {
    Class(Class),
    Any,
}

impl From<u16> for QClass {
    fn from(value: u16) -> Self {
        match value {
            255 => Self::Any,
            other => Self::Class(Class::from(other)),
        }
    }
}

impl From<QClass> for u16 {
    fn from(value: QClass) -> Self {
        match value {
            QClass::Any => 255,
            QClass::Class(c) => u16::from(c),
        }
    }
}

int_enum_with_catchall_u16! {
    pub enum Type {
        A = 1,
        NS = 2,
        MD = 3,
        MF = 4,
        CNAME = 5,
        SOA = 6,
        MB = 7,
        MG = 8,
        MR = 9,
        NULL = 10,
        WKS = 11,
        PTR = 12,
        HINFO = 13,
        MINFO = 14,
        MX = 15,
        TXT = 16,
        AAAA = 28,  // rfc3596
    }
    catch_all = Other
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QType {
    Type(Type),
    AXFR,
    MAILB,
    MAILA,
    All,
}

impl From<u16> for QType {
    fn from(value: u16) -> Self {
        match value {
            252 => Self::AXFR,
            253 => Self::MAILB,
            254 => Self::MAILA,
            255 => Self::All,
            other => Self::Type(Type::from(other)),
        }
    }
}

impl From<QType> for u16 {
    fn from(value: QType) -> Self {
        match value {
            QType::AXFR => 252,
            QType::MAILB => 253,
            QType::MAILA => 254,
            QType::All => 255,
            QType::Type(t) => u16::from(t),
        }
    }
}

#[derive(Debug)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answer: Vec<ResourceRecord>,
    pub authority: Vec<ResourceRecord>,
    pub additional: Vec<ResourceRecord>,
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug, PartialEq)]
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

#[derive(Debug)]
pub struct Question {
    pub qname: Vec<String>,
    pub qtype: QType,
    pub qclass: QClass,
}

impl Question {
    pub fn name(&self) -> String {
        let mut result = self.qname.join(".");
        result.push('.');
        result
    }
}

#[derive(Debug)]
pub struct Label {
    pub value: String,
    pub offsets: Vec<usize>,
}

#[derive(Debug)]
pub struct ResourceRecord {
    pub name: Vec<Label>,
    pub r#type: Type,
    pub class: Class,
    pub ttl: u32,
    pub rdlength: u16,
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    pub fn name(&self) -> String {
        self.name
            .iter()
            .flat_map(|l| [l.value.as_str(), "."])
            .collect::<Vec<_>>()
            .join("")
    }
}
