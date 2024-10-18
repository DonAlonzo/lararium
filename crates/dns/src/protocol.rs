use crate::{Answer, Class, OperationCode, QueryFlags, RecordType, ResponseCode, ResponseFlags};
use bytes::{Buf, BufMut, BytesMut};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct Query {
    pub transaction_id: u16,
    pub flags: QueryFlags,
    pub name: String,
    pub record_type: RecordType,
    pub class: Class,
}

#[derive(Debug, Clone)]
pub struct Response {
    pub transaction_id: u16,
    pub flags: ResponseFlags,
    pub answer_resource_records: u16,
    pub authority_resource_records: u16,
    pub additional_resource_records: u16,
    pub answers: Vec<Answer>,
}

impl Query {
    pub fn encode(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(512);
        buffer.put_u16(self.transaction_id);
        buffer.put_query_flags(&self.flags);
        buffer.put_slice(&self.encode_question());
        buffer.to_vec()
    }

    fn encode_question(&self) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(512);
        let mut name_parts: Vec<&str> = self.name.split('.').collect();
        while !name_parts.is_empty() {
            let label = name_parts.remove(0);
            let label_len = label.len() as u8;
            buffer.put_u8(label_len);
            buffer.put_slice(label.as_bytes());
        }
        buffer.put_u8(0);
        buffer.put_u16(match self.record_type {
            RecordType::A => 0x0001,
            RecordType::Aaaa => 0x001C,
            RecordType::Cname => 0x0005,
            RecordType::Mx => 0x000F,
            RecordType::Ns => 0x0002,
            RecordType::Ptr => 0x000C,
            RecordType::Soa => 0x0006,
            RecordType::Srv => 0x0021,
            RecordType::Txt => 0x0010,
        });
        buffer.put_u16(match self.class {
            Class::Internet => 0x0001,
            Class::Csnet => 0x0002,
            Class::Chaos => 0x0003,
            Class::Hesiod => 0x0004,
            Class::None => 0x00FE,
            Class::Any => 0x00FF,
        });
        buffer.to_vec()
    }

    pub fn decode(buffer: &[u8]) -> Self {
        let mut buffer = &buffer[..];
        if buffer.remaining() < 12 {
            todo!("too short");
        }
        let transaction_id = buffer.get_u16();
        let flags = buffer.get_query_flags();
        buffer.advance(8);
        let name = {
            let mut name = String::new();
            loop {
                let length = buffer.get_u8() as usize;
                if length == 0 {
                    break;
                }
                if !name.is_empty() {
                    name.push('.');
                }
                let label = buffer.copy_to_bytes(length);
                name.push_str(&String::from_utf8_lossy(&label));
            }
            name
        };
        let record_type = match buffer.get_u16() {
            0x0001 => RecordType::A,
            0x001C => RecordType::Aaaa,
            0x0005 => RecordType::Cname,
            0x000F => RecordType::Mx,
            0x0002 => RecordType::Ns,
            0x000C => RecordType::Ptr,
            0x0006 => RecordType::Soa,
            0x0021 => RecordType::Srv,
            0x0010 => RecordType::Txt,
            _ => todo!("unknown record type"),
        };
        let class = match buffer.get_u16() {
            0x0001 => Class::Internet,
            0x0003 => Class::Chaos,
            0x0004 => Class::Hesiod,
            0x00FF => Class::Any,
            _ => todo!("unknown class"),
        };
        Query {
            transaction_id,
            flags,
            name,
            record_type,
            class,
        }
    }
}

impl Response {
    pub fn encode(
        &self,
        query: &Query,
    ) -> Vec<u8> {
        let mut buffer = BytesMut::with_capacity(512);
        let truncated = false;
        buffer.put_u16(self.transaction_id);
        buffer.put_response_flags(&self.flags, truncated);
        buffer.put_u16(1); // number of questions
        buffer.put_u16(self.answer_resource_records);
        buffer.put_u16(self.authority_resource_records);
        buffer.put_u16(self.additional_resource_records);
        buffer.put_slice(&query.encode_question());
        let mut name_offsets = HashMap::new();
        for answer in &self.answers {
            answer.encode(&mut buffer, &mut name_offsets);
        }
        buffer.to_vec()
    }
}

impl Answer {
    fn encode(
        &self,
        buffer: &mut BytesMut,
        name_offsets: &mut HashMap<String, usize>,
    ) {
        let mut name_parts: Vec<&str> = self.name.split('.').collect();
        while !name_parts.is_empty() {
            let current_name = name_parts.join(".");
            if let Some(&offset) = name_offsets.get(&current_name) {
                let pointer = 0xC000 | (offset as u16);
                buffer.put_u16(pointer);
                return;
            }
            name_offsets.insert(current_name.clone(), buffer.len());
            let label = name_parts.remove(0);
            let label_len = label.len();
            if label_len > 63 {
                todo!("label exceeds 63 characters");
            }
            buffer.put_u8(label_len as u8);
            buffer.put_slice(label.as_bytes());
        }
        buffer.put_u8(0);
        buffer.put_u16(match self.record_type {
            RecordType::A => 0x01,
            RecordType::Aaaa => 0x1C,
            RecordType::Cname => 0x05,
            RecordType::Mx => 0x0F,
            RecordType::Ns => 0x02,
            RecordType::Ptr => 0x0C,
            RecordType::Soa => 0x06,
            RecordType::Srv => 0x21,
            RecordType::Txt => 0x10,
        });
        buffer.put_u16(match self.class {
            Class::Internet => 0x01,
            Class::Csnet => 0x02,
            Class::Chaos => 0x03,
            Class::Hesiod => 0x04,
            Class::None => 0xFE,
            Class::Any => 0xFF,
        });
        buffer.put_u32(self.ttl);
        buffer.put_u16(self.data.len() as u16);
        buffer.put_slice(&self.data);
    }
}

trait BufExt {
    fn get_query_flags(&mut self) -> QueryFlags;
    fn get_response_flags(&mut self) -> ResponseFlags;
}

trait BufMutExt {
    fn put_query_flags(
        &mut self,
        flags: &QueryFlags,
    );
    fn put_response_flags(
        &mut self,
        flags: &ResponseFlags,
        truncated: bool,
    );
}

impl<T: Buf> BufExt for T {
    fn get_query_flags(&mut self) -> QueryFlags {
        let flags = self.get_u16();
        let operation_code = match (flags >> 11) & 0b1111 {
            0b0000 => OperationCode::StandardQuery,
            0b0001 => OperationCode::InverseQuery,
            0b0010 => OperationCode::Status,
            _ => todo!("unknown operation code"),
        };
        let truncated = ((flags >> 9) & 1) != 0;
        let recursion_desired = ((flags >> 8) & 1) != 0;
        QueryFlags {
            operation_code,
            truncated,
            recursion_desired,
        }
    }

    fn get_response_flags(&mut self) -> ResponseFlags {
        todo!();
    }
}

impl<T: BufMut> BufMutExt for T {
    fn put_query_flags(
        &mut self,
        query_flags: &QueryFlags,
    ) {
        let mut flags = 0;
        flags |= match query_flags.operation_code {
            OperationCode::StandardQuery => 0b0000 << 11,
            OperationCode::InverseQuery => 0b0001 << 11,
            OperationCode::Status => 0b0010 << 11,
        };
        if query_flags.truncated {
            flags |= 1 << 9;
        }
        if query_flags.recursion_desired {
            flags |= 1 << 8;
        }
        self.put_u16(flags);
    }

    fn put_response_flags(
        &mut self,
        response_flags: &ResponseFlags,
        truncated: bool,
    ) {
        let mut flags = 0;
        flags |= 1 << 15;
        flags |= match response_flags.operation_code {
            OperationCode::StandardQuery => 0b0000 << 11,
            OperationCode::InverseQuery => 0b0001 << 11,
            OperationCode::Status => 0b0010 << 11,
        };
        if response_flags.authoritative_answer {
            flags |= 1 << 10;
        }
        if truncated {
            flags |= 1 << 9;
        }
        if response_flags.recursion_desired {
            flags |= 1 << 8;
        }
        if response_flags.recursion_available {
            flags |= 1 << 7;
        }
        flags |= match response_flags.response_code {
            ResponseCode::NoError => 0b0000,
            ResponseCode::Malformed => 0b0001,
            ResponseCode::ServerFailure => 0b0010,
            ResponseCode::NonExistentDomain => 0b0011,
            ResponseCode::NotImplemented => 0b0100,
            ResponseCode::Refused => 0b0101,
        };
        self.put_u16(flags);
    }
}
