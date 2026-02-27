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

use std::time::Instant;

use crate::{
    NamingError, UnexpectedValueError,
    protocol::{LABEL_LEN_MAX, NAME_LEN_MAX},
};

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
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, PartialEq)]
pub struct Message {
    pub header: Header,
    pub questions: Vec<Question>,
    pub answers: Vec<ResourceRecord>,
    pub authority: Vec<ResourceRecord>,
    pub additional: Vec<ResourceRecord>,
}

impl Message {
    pub fn serialize(&self) -> Vec<u8> {
        let mut payload = Vec::new();
        payload.append(&mut self.header.serialize());
        payload.extend_from_slice(&(self.questions.len() as u16).to_be_bytes());
        payload.extend_from_slice(&(self.answers.len() as u16).to_be_bytes());
        payload.extend_from_slice(&(self.authority.len() as u16).to_be_bytes());
        payload.extend_from_slice(&(self.additional.len() as u16).to_be_bytes());
        self.questions
            .iter()
            .for_each(|q| payload.append(&mut q.serialize()));
        self.answers
            .iter()
            .for_each(|rr| payload.append(&mut rr.serialize()));
        self.authority
            .iter()
            .for_each(|rr| payload.append(&mut rr.serialize()));
        self.additional
            .iter()
            .for_each(|rr| payload.append(&mut rr.serialize()));
        payload
    }
}

#[derive(Debug, PartialEq)]
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
}

impl Header {
    fn serialize(&self) -> Vec<u8> {
        let mut chunk = Vec::new();
        chunk.extend_from_slice(&self.id.to_be_bytes());
        let mut flags = 0_u16;
        flags |= u16::from(self.query_response) << 15;
        flags |= u16::from(self.opcode) << 11;
        flags |= u16::from(self.authoritative_answer) << 10;
        flags |= u16::from(self.truncation) << 9;
        flags |= u16::from(self.recursion_desired) << 8;
        flags |= u16::from(self.recursion_available) << 7;
        flags |= u16::from(self.reserved) << 6;
        flags |= u16::from(self.authentic_data) << 5;
        flags |= u16::from(self.checking_disabled) << 4;
        flags |= u16::from(self.response_code);
        chunk.extend_from_slice(&flags.to_be_bytes());
        chunk
    }
}

pub type Id = u16;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum QueryResponse {
    Query,
    Response,
}

impl TryFrom<u8> for QueryResponse {
    type Error = UnexpectedValueError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => QueryResponse::Query,
            1 => QueryResponse::Response,
            other => return Err(UnexpectedValueError(format!("This value should not happen: {other}"))),
        })
    }
}

impl From<QueryResponse> for u16 {
    fn from(value: QueryResponse) -> Self {
        match value {
            QueryResponse::Query => 0,
            QueryResponse::Response => 1,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum OpCode {
    Query,
    InverseQuery,
    Status,
    Reserved(u8),
}

impl TryFrom<u8> for OpCode {
    type Error = UnexpectedValueError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => OpCode::Query,
            1 => OpCode::InverseQuery,
            2 => OpCode::Status,
            x @ 3..=15 => OpCode::Reserved(x),
            other => return Err(UnexpectedValueError(format!("This value should not happen: {other}"))),
        })
    }
}

impl From<OpCode> for u16 {
    fn from(value: OpCode) -> Self {
        match value {
            OpCode::Query => 0,
            OpCode::InverseQuery => 1,
            OpCode::Status => 2,
            OpCode::Reserved(v) => v as u16,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    type Error = UnexpectedValueError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => ResponseCode::NoErrorCondition,
            1 => ResponseCode::FormatError,
            2 => ResponseCode::ServerFailure,
            3 => ResponseCode::NameError,
            4 => ResponseCode::NotImplemented,
            5 => ResponseCode::Refused,
            x @ 6..=15 => ResponseCode::Reserved(x),
            other => return Err(UnexpectedValueError(format!("This value should not happen: {other}"))),
        })
    }
}

impl From<ResponseCode> for u16 {
    fn from(value: ResponseCode) -> Self {
        match value {
            ResponseCode::NoErrorCondition => 0,
            ResponseCode::FormatError => 1,
            ResponseCode::ServerFailure => 2,
            ResponseCode::NameError => 3,
            ResponseCode::NotImplemented => 4,
            ResponseCode::Refused => 5,
            ResponseCode::Reserved(r) => r as u16,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Question {
    pub qname: Vec<String>,
    pub qtype: QType,
    pub qclass: QClass,
}

impl Question {
    pub fn new(name: &str, qclass: QClass, qtype: QType) -> Result<Question, NamingError> {
        if name.len() > NAME_LEN_MAX - 1 {
            return Err(NamingError::InvalidNameSize { name: name.into() });
        }
        let qname = name.split('.').map(String::from).collect::<Vec<_>>();
        for label in &qname {
            let label_len = label.len();
            if label_len > LABEL_LEN_MAX {
                return Err(NamingError::InvalidLabelSize { label: label.into() });
            }
        }
        Ok(Question {
            qname,
            qtype,
            qclass,
        })
    }
    pub fn name(&self) -> String {
        self.qname.join(".")
    }

    fn serialize(&self) -> Vec<u8> {
        let mut chunk = Vec::new();
        for label in &self.qname {
            chunk.push(label.len() as u8);
            chunk.extend_from_slice(&label.as_bytes())
        }
        chunk.push(0u8);
        chunk.extend_from_slice(&u16::from(self.qtype).to_be_bytes());
        chunk.extend_from_slice(&u16::from(self.qclass).to_be_bytes());
        chunk
    }
}

#[derive(Debug, Clone)]
pub struct Label {
    pub value: String,
    pub offsets: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct CompressedName(pub Vec<Label>);

impl CompressedName {
    pub fn name(&self) -> String {
        self.0
            .iter()
            .map(|v| v.value.clone())
            .collect::<Vec<_>>()
            .join(".")
    }
    pub fn labels(&self) -> Vec<String> {
        self.0.iter().map(|v| v.value.clone()).collect::<Vec<_>>()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResourceRecord {
    pub name: Vec<String>,
    pub r#type: Type,
    pub class: Class,
    pub ttl: Instant,
    pub rdata: Vec<u8>,
}

impl ResourceRecord {
    pub fn name(&self) -> String {
        self.name.join(".")
    }
    pub fn serialize(&self) -> Vec<u8> {
        let mut chunk = Vec::new();
        for label in &self.name {
            chunk.push(label.len() as u8);
            chunk.extend_from_slice(&label.as_bytes())
        }
        chunk.push(0u8);
        chunk.extend_from_slice(&u16::from(self.r#type).to_be_bytes());
        chunk.extend_from_slice(&u16::from(self.class).to_be_bytes());
        chunk.extend_from_slice(&((self.ttl - Instant::now()).as_secs() as u32).to_be_bytes());
        chunk.extend_from_slice(&(self.rdata.len() as u16).to_be_bytes());
        chunk.extend_from_slice(&self.rdata);
        chunk
    }
}

#[cfg(test)]
mod tests {
    use crate::protocol::parser::parse;

    use super::*;

    #[test]
    fn test_serialize_deserialize_full() {
        let qmessage = Message {
            header: Header {
                id: 42,
                query_response: QueryResponse::Query,
                opcode: OpCode::Query,
                authoritative_answer: false,
                truncation: false,
                recursion_desired: true,
                recursion_available: false,
                reserved: false,
                authentic_data: false,
                checking_disabled: true,
                response_code: ResponseCode::NoErrorCondition,
            },
            questions: vec![
                Question::new(
                    "www.alea.net",
                    QClass::Class(Class::Internet),
                    QType::Type(Type::A),
                )
                .unwrap(),
            ],
            answers: Vec::new(),
            authority: Vec::new(),
            additional: Vec::new(),
        };
        let bin_message = qmessage.serialize();
        assert_eq!(qmessage, parse(&bin_message).unwrap());
    }
}
