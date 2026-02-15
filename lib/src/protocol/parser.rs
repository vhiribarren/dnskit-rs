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

use std::error::Error;

use tracing::trace;

use crate::protocol::message::{
    Header, Message, OpCode, QClass, QType, QueryResponse, Question, ResponseCode,
};

pub fn parse(mut buffer: &[u8]) -> Result<Message, Box<dyn Error>> {
    let header = parse_header(&mut buffer)?;
    let questions = (0..header.qd_count)
        .map(|_| parse_question(buffer))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Message { header, questions })
}

fn parse_header(buffer: &mut &[u8]) -> Result<Header, Box<dyn Error>> {
    let id = consume_u16(buffer)?;
    let query_response = QueryResponse::try_from(extract_from_u8(buffer[0], 7, 7))?;
    let opcode = OpCode::try_from(extract_from_u8(buffer[0], 3, 6))?;
    let authoritative_answer: bool = bool_from_u8(buffer[0], 2);
    let truncation: bool = bool_from_u8(buffer[0], 1);
    let recursion_desired: bool = bool_from_u8(buffer[0], 0);
    consume(buffer, 1);
    let recursion_available = bool_from_u8(buffer[0], 7);
    let reserved = bool_from_u8(buffer[0], 6);
    let authentic_data = bool_from_u8(buffer[0], 5);
    let checking_disabled = bool_from_u8(buffer[0], 4);
    let response_code = ResponseCode::try_from(extract_from_u8(buffer[0], 0, 3))?;
    consume(buffer, 1);
    let qd_count = consume_u16(buffer)?;
    let an_count = consume_u16(buffer)?;
    let ns_count = consume_u16(buffer)?;
    let ar_count = consume_u16(buffer)?;
    Ok(Header {
        id,
        query_response,
        opcode,
        authoritative_answer,
        truncation,
        recursion_desired,
        recursion_available,
        reserved,
        authentic_data,
        checking_disabled,
        response_code,
        qd_count,
        an_count,
        ns_count,
        ar_count,
    })
}

fn parse_question(mut buffer: &[u8]) -> Result<Question, Box<dyn Error>> {
    let mut qname = Vec::new();
    while let Ok(length) = consume_u8(&mut buffer) {
        trace!(?qname);
        if length == 0 {
            break;
        }
        qname.push(String::from_utf8(
            consume_slice(&mut buffer, length as usize)?.into(),
        )?);
    }
    let qtype = QType::from(consume_u16(&mut buffer)?);
    let qclass = QClass::from(consume_u16(&mut buffer)?);
    Ok(Question {
        qname,
        qtype,
        qclass,
    })
}

#[inline(always)]
fn consume(buffer: &mut &[u8], count: usize) {
    *buffer = &buffer[count..];
}

fn consume_slice<'a>(buffer: &mut &'a [u8], count: usize) -> Result<&'a [u8], Box<dyn Error>> {
    let result = &buffer[..count];
    consume(buffer, count);
    Ok(result)
}

fn consume_u8(buffer: &mut &[u8]) -> Result<u8, Box<dyn Error>> {
    let val = buffer[0];
    consume(buffer, 1);
    Ok(val)
}

fn consume_u16(buffer: &mut &[u8]) -> Result<u16, Box<dyn Error>> {
    let val = u16::from_be_bytes(buffer[..2].try_into()?);
    consume(buffer, 2);
    Ok(val)
}

fn extract_from_u8(buffer: u8, min_idx: u8, max_idx: u8) -> u8 {
    assert!(min_idx <= max_idx);
    assert!(max_idx < 8);
    (buffer >> min_idx) & ((1 << (max_idx - min_idx + 1)) - 1)
}

fn bool_from_u8(buffer: u8, index: u8) -> bool {
    (buffer >> index) & 0x01 == 1
}

#[cfg(test)]
mod tests {
    use crate::protocol::allocate_udp_recv_buffer;

    use super::*;

    const PAYLOAD: &'static str =
        "33B80120000100000000000104616C6561036E657400000100010000291000000000000000";

    #[test]
    fn test_parse_header() {
        let mut recv_buffer = allocate_udp_recv_buffer();
        hex::decode_to_slice(PAYLOAD, &mut recv_buffer[..PAYLOAD.len() / 2]).unwrap();
        let message = parse(&recv_buffer).unwrap();
        let header = message.header;

        assert_eq!(header.id, 13240);
        assert_eq!(header.query_response, QueryResponse::Query);
        assert_eq!(header.opcode, OpCode::Query);
        assert_eq!(header.authoritative_answer, false);
        assert_eq!(header.truncation, false);
        assert_eq!(header.recursion_available, false);
        assert_eq!(header.recursion_desired, true);
        assert_eq!(header.authentic_data, true);
        assert_eq!(header.checking_disabled, false);
        assert_eq!(header.response_code, ResponseCode::NoErrorCondition);
        assert_eq!(header.qd_count, 1);
        assert_eq!(header.an_count, 0);
        assert_eq!(header.ns_count, 0);
        assert_eq!(header.ar_count, 1);
    }

    #[test]
    fn test_parse_question() {
        let mut recv_buffer = allocate_udp_recv_buffer();
        hex::decode_to_slice(PAYLOAD, &mut recv_buffer[..PAYLOAD.len() / 2]).unwrap();
        let message = parse(&recv_buffer).unwrap();
        assert_eq!(message.questions.len(), 1);
        let question = message.questions.iter().next().unwrap();

        assert_eq!(&question.name(), "alea.net");
        assert_eq!(question.qtype, QType::A);
        assert_eq!(question.qclass, QClass::Internet);
    }
}
