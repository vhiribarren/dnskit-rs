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

use crate::{ParseError, TextError, protocol::NAME_LEN_MAX};

pub struct TXT {
    entries: Vec<String>,
}

impl TXT {
    pub fn with_single(entry: &str) -> Result<Self, TextError> {
        if entry.len() > NAME_LEN_MAX - 1 {
            return Err(TextError::InvalidNameSize { text: entry.into() });
        }
        Ok(Self {
            entries: vec![entry.to_string()],
        })
    }

    pub fn with_multiple(entries: Vec<String>) -> Result<Self, TextError> {
        for entry in &entries {
            if entry.len() > NAME_LEN_MAX - 1 {
                return Err(TextError::InvalidNameSize { text: entry.into() });
            }
        }
        Ok(Self { entries })
    }

    pub fn parse(mut buffer: &[u8]) -> Result<Self, ParseError> {
        let mut entries = Vec::new();
        while let buff_len = buffer.len()
            && buff_len > 0
        {
            let len = buffer[0] as usize;
            if buff_len < len + 1 {
                return Err(ParseError::InvalidSliceSize);
            }
            entries.push(String::from_utf8_lossy(&buffer[1..1 + len]).to_string());
            buffer = &buffer[1 + len..];
        }
        Ok(Self { entries })
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut result = Vec::new();
        for entry in &self.entries {
            assert!(entry.len() < 256);
            result.push(entry.len() as u8);
            result.extend_from_slice(entry.as_bytes());
        }
        result
    }

    pub fn text(&self) -> String {
        self.entries.join("\n")
    }
}
