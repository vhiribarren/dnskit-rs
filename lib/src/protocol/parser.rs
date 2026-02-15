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

use crate::protocol::message::{Header, Message, OpCode, QueryResponse, ResponseCode};

pub fn parse(buffer: &[u8]) -> Result<Message, Box<dyn Error>> {
    let header = parse_header(&buffer)?;
    Ok(Message { header })
}

fn parse_header(mut buffer: &[u8]) -> Result<Header, Box<dyn Error>> {
    let id = consume_u16(&mut buffer)?;
    let query_response = QueryResponse::try_from(extract_from_u8(buffer[0], 7, 7))?;
    let opcode = OpCode::try_from(extract_from_u8(buffer[0], 3, 6))?;
    let authoritative_answer: bool = bool_from_u8(buffer[0], 2);
    let truncation: bool = bool_from_u8(buffer[0], 1);
    let recursion_desired: bool = bool_from_u8(buffer[0], 0);
    consume(&mut buffer, 1);
    let recursion_available = bool_from_u8(buffer[0], 7);
    let reserved = bool_from_u8(buffer[0], 6);
    let authentic_data = bool_from_u8(buffer[0], 5);
    let checking_disabled = bool_from_u8(buffer[0], 4);
    let response_code = ResponseCode::try_from(extract_from_u8(buffer[0], 0, 3))?;
    consume(&mut buffer, 1);
    let qd_count = consume_u16(&mut buffer)?;
    let an_count = consume_u16(&mut buffer)?;
    let ns_count = consume_u16(&mut buffer)?;
    let ar_count = consume_u16(&mut buffer)?;
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

#[inline(always)]
fn consume(buffer: &mut &[u8], index: usize) {
    *buffer = &buffer[index..];
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
