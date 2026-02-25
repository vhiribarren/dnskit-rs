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

use std::{error::Error, time::{Duration, Instant}};

use crate::protocol::{
    LABEL_LEN_MAX, NAME_LEN_MAX,
    message::{
        CompressedName, Header, Label, Message, OpCode, QClass, QType, QueryResponse, Question,
        ResourceRecord, ResponseCode,
    },
};

pub fn parse(mut buffer: &[u8]) -> Result<Message, Box<dyn Error>> {
    let full_payload = buffer;
    let FullHeader {
        header,
        qd_count,
        an_count,
        ns_count,
        ar_count,
    } = parse_header(&mut buffer)?;
    let questions = (0..qd_count)
        .map(|_| parse_question(&mut buffer))
        .collect::<Result<Vec<_>, _>>()?;
    let answer = (0..an_count)
        .map(|_| parse_resource_record(&mut buffer, full_payload))
        .collect::<Result<Vec<_>, _>>()?;
    let authority = (0..ns_count)
        .map(|_| parse_resource_record(&mut buffer, full_payload))
        .collect::<Result<Vec<_>, _>>()?;
    let additional = (0..ar_count)
        .map(|_| parse_resource_record(&mut buffer, full_payload))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Message {
        header,
        questions,
        answer,
        authority,
        additional,
    })
}

struct FullHeader {
    header: Header,
    qd_count: u16,
    an_count: u16,
    ns_count: u16,
    ar_count: u16,
}

fn parse_header(buffer: &mut &[u8]) -> Result<FullHeader, Box<dyn Error>> {
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
    let header = Header {
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
    };
    Ok(FullHeader {
        header,
        qd_count,
        an_count,
        ns_count,
        ar_count,
    })
}

fn parse_question(buffer: &mut &[u8]) -> Result<Question, Box<dyn Error>> {
    let mut qname = Vec::new(); // TODO Ensure name and lables are ASCII and within max level ranges
    while let Ok(length) = consume_u8(buffer) {
        if length == 0 {
            break;
        }
        qname.push(String::from_utf8(
            consume_slice(buffer, length as usize)?.into(),
        )?);
    }
    let qtype = QType::from(consume_u16(buffer)?);
    let qclass = QClass::from(consume_u16(buffer)?);
    Ok(Question {
        qname,
        qtype,
        qclass,
    })
}

fn parse_resource_record(
    buffer: &mut &[u8],
    full_buffer: &[u8],
) -> Result<ResourceRecord, Box<dyn Error>> {
    let name = parse_resource_record_name(buffer, full_buffer)?;
    let r#type = consume_u16(buffer)?.into();
    let class = consume_u16(buffer)?.into();
    let ttl = Instant::now() + Duration::from_secs(consume_u32(buffer)? as u64);
    let rdlength = consume_u16(buffer)?;
    let rdata = consume_slice(buffer, rdlength as usize)?.into();
    Ok(ResourceRecord {
        name: name.labels(),
        r#type,
        class,
        ttl,
        rdata,
    })
}

fn parse_resource_record_name(
    buffer: &mut &[u8],
    full_buffer: &[u8],
) -> Result<CompressedName, Box<dyn Error>> {
    let mut name_len = 0;
    let mut name = Vec::new();
    let mut marker;
    let mut offsets = Vec::new();

    loop {
        marker = (buffer[0] & 0b11000000) >> 6;
        if marker == 0b11 {
            offsets.push(0x3FFF & consume_u16(buffer)? as usize);
            break;
        }
        if marker != 0b00 {
            return Err("error".into());
        }
        let length = consume_u8(buffer)? as usize;
        if length == 0 {
            if name_len + 1 > NAME_LEN_MAX {
                return Err("error".into());
            }
            return Ok(CompressedName(name));
        }
        if length > LABEL_LEN_MAX {
            return Err("error".into());
        }
        name_len += 1 + length;
        if name_len > NAME_LEN_MAX {
            return Err("error".into());
        }
        name.push(Label {
            offsets: Vec::new(),
            value: String::from_utf8(consume_slice(buffer, length as usize)?.into())?,
        });
    }

    assert_eq!(marker, 0b11);
    assert_eq!(offsets.len(), 1);
    let mut offset = *offsets.last().unwrap();
    // TODO Should also check if there is a cycle
    // TODO Should ensure we do not go outside of bounds
    loop {
        marker = (full_buffer[offset] & 0b11000000) >> 6;
        match marker {
            0b11 => {
                offset = 0x3FFF & peek_u16(&full_buffer[offset..offset + 2])? as usize;
                offsets.push(offset);
            }
            0b00 => {
                let length = full_buffer[offset] as usize;
                if length == 0 {
                    if name_len + 1 > NAME_LEN_MAX {
                        return Err("error".into());
                    }
                    return Ok(CompressedName(name));
                }
                if length > LABEL_LEN_MAX {
                    return Err("error".into());
                }
                name_len += length + 1;
                if name_len > NAME_LEN_MAX {
                    return Err("error".into());
                }
                name.push(Label {
                    offsets,
                    value: String::from_utf8(full_buffer[offset + 1..offset + 1 + length].into())?,
                });
                offsets = Vec::new();
                offset += length + 1;
            }
            _ => {
                return Err("error".into());
            }
        }
    }
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

fn peek_u16(buffer: &[u8]) -> Result<u16, Box<dyn Error>> {
    Ok(u16::from_be_bytes(buffer[..2].try_into()?))
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

fn consume_u32(buffer: &mut &[u8]) -> Result<u32, Box<dyn Error>> {
    let val = u32::from_be_bytes(buffer[..4].try_into()?);
    consume(buffer, 4);
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
    use crate::protocol::{
        allocate_udp_recv_buffer,
        message::{Class, Type},
    };

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
        assert_eq!(message.questions.len(), 1);
        assert_eq!(message.answer.len(), 0);
        assert_eq!(message.authority.len(), 0);
        assert_eq!(message.additional.len(), 1);
    }

    #[test]
    fn test_parse_question() {
        let mut recv_buffer = allocate_udp_recv_buffer();
        hex::decode_to_slice(PAYLOAD, &mut recv_buffer[..PAYLOAD.len() / 2]).unwrap();
        let message = parse(&recv_buffer).unwrap();
        assert_eq!(message.questions.len(), 1);
        let question = message.questions.iter().next().unwrap();

        assert_eq!(&question.name(), "alea.net");
        assert_eq!(question.qtype, QType::Type(Type::A));
        assert_eq!(question.qclass, QClass::Class(Class::Internet));
    }

    #[test]
    fn test_parse_rr_name_no_offsets() {
        let labels = vec!["www", "alea", "net"];
        let result = "www.alea.net";
        let mut buffer = Vec::new();
        for label in &labels {
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        buffer.push(0);
        let parse_result = parse_resource_record_name(&mut &buffer[..], &buffer).unwrap();
        assert_eq!(parse_result.name(), result);
    }

    #[test]
    fn test_parse_rr_name_offsets_immediate() {
        let labels = vec!["www", "alea", "net"];
        let result = "www.alea.net";
        let mut buffer = Vec::new();
        buffer.extend_from_slice(&0xC003_u16.to_be_bytes()); // Starting at offset 3
        buffer.push(0); // padding    
        for label in &labels {
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        buffer.push(0);
        let parse_result = parse_resource_record_name(&mut &buffer[..], &buffer).unwrap();
        assert_eq!(parse_result.name(), result);
    }

    #[test]
    #[ignore]
    fn test_parse_rr_name_offsets_one() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn test_parse_rr_name_offsets_two() {
        unimplemented!()
    }

    #[test]
    #[ignore]
    fn test_parse_rr_name_offsets_loop() {
        unimplemented!()
    }

    #[test]
    fn test_parse_rr_name_label_max() {
        let long_label = "a".repeat(63);
        let labels = vec![&long_label, "alea", "net"];
        let result = long_label.clone() + ".alea.net";
        let mut buffer = Vec::new();
        for label in &labels {
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        buffer.push(0);
        let parse_result = parse_resource_record_name(&mut &buffer[..], &buffer).unwrap();
        assert_eq!(parse_result.name(), result);
    }

    #[test]
    #[should_panic]
    fn test_parse_rr_name_label_oversize() {
        let long_label = "a".repeat(64);
        let labels = vec![&long_label, "alea", "net"];
        let mut buffer = Vec::new();
        for label in &labels {
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        buffer.push(0);
        parse_resource_record_name(&mut &buffer[..], &buffer).unwrap();
    }

    #[test]
    fn test_parse_rr_name_total_max() {
        let long_label = "a".repeat(49);
        let labels = vec![
            &long_label,
            &long_label,
            &long_label,
            &long_label,
            &long_label,
            "123",
        ];
        let mut buffer = Vec::new();
        for label in &labels {
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        buffer.push(0);
        assert_eq!(buffer.len(), 255);
        let result = parse_resource_record_name(&mut &buffer[..], &buffer).unwrap();
        assert_eq!(result.name().len(), 253);
    }

    #[test]
    #[should_panic]
    fn test_parse_rr_name_total_oversize() {
        let long_label = "a".repeat(49);
        let labels = vec![
            &long_label,
            &long_label,
            &long_label,
            &long_label,
            &long_label,
            "1234",
        ];
        let mut buffer = Vec::new();
        for label in &labels {
            buffer.push(label.len() as u8);
            buffer.extend_from_slice(label.as_bytes());
        }
        assert_eq!(buffer.len(), 256);
        buffer.push(0);
        parse_resource_record_name(&mut &buffer[..], &buffer).unwrap();
    }
}
