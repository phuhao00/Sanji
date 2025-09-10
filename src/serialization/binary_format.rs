//! 二进制序列化器

use super::{Serializer, SerializationContext};
use serde::{Deserialize, Serialize};

/// 二进制序列化器
pub struct BinarySerializer {
    use_compact_format: bool,
}

impl BinarySerializer {
    pub fn new() -> Self {
        Self {
            use_compact_format: true,
        }
    }

    pub fn with_compact_format(mut self, compact: bool) -> Self {
        self.use_compact_format = compact;
        self
    }
}

impl Default for BinarySerializer {
    fn default() -> Self {
        Self::new()
    }
}

impl Serializer for BinarySerializer {
    type Error = Box<dyn std::error::Error + Send + Sync>;

    fn serialize<T: Serialize>(&self, data: &T, _context: &SerializationContext) -> Result<Vec<u8>, Self::Error> {
        // 使用bincode进行二进制序列化
        let result = if self.use_compact_format {
            bincode::serialize(data)?
        } else {
            // 使用标准配置
            let config = bincode::DefaultOptions::new()
                .with_big_endian()
                .with_fixint_encoding();
            config.serialize(data)?
        };
        
        Ok(result)
    }

    fn deserialize<T: for<'de> Deserialize<'de>>(&self, data: &[u8], _context: &SerializationContext) -> Result<T, Self::Error> {
        let result = if self.use_compact_format {
            bincode::deserialize(data)?
        } else {
            let config = bincode::DefaultOptions::new()
                .with_big_endian()
                .with_fixint_encoding();
            config.deserialize(data)?
        };
        
        Ok(result)
    }
}

/// 二进制序列化工具函数
pub mod binary_utils {
    use super::*;
    use crate::EngineResult;
    use std::io::{Read, Write, Cursor};
    use byteorder::{LittleEndian, BigEndian, ReadBytesExt, WriteBytesExt};

    /// 序列化为二进制数据
    pub fn to_binary<T: Serialize>(data: &T) -> EngineResult<Vec<u8>> {
        let bytes = bincode::serialize(data)?;
        Ok(bytes)
    }

    /// 从二进制数据反序列化
    pub fn from_binary<T: for<'de> Deserialize<'de>>(data: &[u8]) -> EngineResult<T> {
        let result = bincode::deserialize(data)?;
        Ok(result)
    }

    /// 序列化为紧凑二进制格式
    pub fn to_compact_binary<T: Serialize>(data: &T) -> EngineResult<Vec<u8>> {
        let config = bincode::DefaultOptions::new()
            .with_little_endian()
            .with_varint_encoding()
            .with_limit(u64::MAX);
        let bytes = config.serialize(data)?;
        Ok(bytes)
    }

    /// 从紧凑二进制格式反序列化
    pub fn from_compact_binary<T: for<'de> Deserialize<'de>>(data: &[u8]) -> EngineResult<T> {
        let config = bincode::DefaultOptions::new()
            .with_little_endian()
            .with_varint_encoding()
            .with_limit(u64::MAX);
        let result = config.deserialize(data)?;
        Ok(result)
    }

    /// 二进制写入器
    pub struct BinaryWriter {
        buffer: Vec<u8>,
        cursor: Cursor<Vec<u8>>,
    }

    impl BinaryWriter {
        pub fn new() -> Self {
            Self {
                buffer: Vec::new(),
                cursor: Cursor::new(Vec::new()),
            }
        }

        pub fn with_capacity(capacity: usize) -> Self {
            Self {
                buffer: Vec::with_capacity(capacity),
                cursor: Cursor::new(Vec::with_capacity(capacity)),
            }
        }

        /// 写入字节
        pub fn write_bytes(&mut self, data: &[u8]) -> EngineResult<()> {
            self.cursor.write_all(data)?;
            Ok(())
        }

        /// 写入字符串
        pub fn write_string(&mut self, s: &str) -> EngineResult<()> {
            let bytes = s.as_bytes();
            self.write_u32_le(bytes.len() as u32)?;
            self.write_bytes(bytes)?;
            Ok(())
        }

        /// 写入32位无符号整数（小端序）
        pub fn write_u32_le(&mut self, value: u32) -> EngineResult<()> {
            self.cursor.write_u32::<LittleEndian>(value)?;
            Ok(())
        }

        /// 写入32位无符号整数（大端序）
        pub fn write_u32_be(&mut self, value: u32) -> EngineResult<()> {
            self.cursor.write_u32::<BigEndian>(value)?;
            Ok(())
        }

        /// 写入64位无符号整数（小端序）
        pub fn write_u64_le(&mut self, value: u64) -> EngineResult<()> {
            self.cursor.write_u64::<LittleEndian>(value)?;
            Ok(())
        }

        /// 写入浮点数（小端序）
        pub fn write_f32_le(&mut self, value: f32) -> EngineResult<()> {
            self.cursor.write_f32::<LittleEndian>(value)?;
            Ok(())
        }

        /// 写入双精度浮点数（小端序）
        pub fn write_f64_le(&mut self, value: f64) -> EngineResult<()> {
            self.cursor.write_f64::<LittleEndian>(value)?;
            Ok(())
        }

        /// 写入布尔值
        pub fn write_bool(&mut self, value: bool) -> EngineResult<()> {
            self.cursor.write_u8(if value { 1 } else { 0 })?;
            Ok(())
        }

        /// 获取写入的数据
        pub fn into_bytes(self) -> Vec<u8> {
            self.cursor.into_inner()
        }

        /// 获取当前位置
        pub fn position(&self) -> u64 {
            self.cursor.position()
        }

        /// 设置位置
        pub fn set_position(&mut self, pos: u64) {
            self.cursor.set_position(pos);
        }
    }

    /// 二进制读取器
    pub struct BinaryReader {
        cursor: Cursor<Vec<u8>>,
    }

    impl BinaryReader {
        pub fn new(data: Vec<u8>) -> Self {
            Self {
                cursor: Cursor::new(data),
            }
        }

        /// 读取字节
        pub fn read_bytes(&mut self, len: usize) -> EngineResult<Vec<u8>> {
            let mut buffer = vec![0u8; len];
            self.cursor.read_exact(&mut buffer)?;
            Ok(buffer)
        }

        /// 读取字符串
        pub fn read_string(&mut self) -> EngineResult<String> {
            let len = self.read_u32_le()? as usize;
            let bytes = self.read_bytes(len)?;
            let s = String::from_utf8(bytes)?;
            Ok(s)
        }

        /// 读取32位无符号整数（小端序）
        pub fn read_u32_le(&mut self) -> EngineResult<u32> {
            let value = self.cursor.read_u32::<LittleEndian>()?;
            Ok(value)
        }

        /// 读取32位无符号整数（大端序）
        pub fn read_u32_be(&mut self) -> EngineResult<u32> {
            let value = self.cursor.read_u32::<BigEndian>()?;
            Ok(value)
        }

        /// 读取64位无符号整数（小端序）
        pub fn read_u64_le(&mut self) -> EngineResult<u64> {
            let value = self.cursor.read_u64::<LittleEndian>()?;
            Ok(value)
        }

        /// 读取浮点数（小端序）
        pub fn read_f32_le(&mut self) -> EngineResult<f32> {
            let value = self.cursor.read_f32::<LittleEndian>()?;
            Ok(value)
        }

        /// 读取双精度浮点数（小端序）
        pub fn read_f64_le(&mut self) -> EngineResult<f64> {
            let value = self.cursor.read_f64::<LittleEndian>()?;
            Ok(value)
        }

        /// 读取布尔值
        pub fn read_bool(&mut self) -> EngineResult<bool> {
            let value = self.cursor.read_u8()?;
            Ok(value != 0)
        }

        /// 获取当前位置
        pub fn position(&self) -> u64 {
            self.cursor.position()
        }

        /// 设置位置
        pub fn set_position(&mut self, pos: u64) {
            self.cursor.set_position(pos);
        }

        /// 检查是否到达末尾
        pub fn is_eof(&self) -> bool {
            self.cursor.position() >= self.cursor.get_ref().len() as u64
        }

        /// 获取剩余字节数
        pub fn remaining(&self) -> usize {
            (self.cursor.get_ref().len() as u64 - self.cursor.position()) as usize
        }
    }

    /// 自定义二进制格式
    pub struct CustomBinaryFormat {
        magic_number: u32,
        version: u16,
        flags: u16,
    }

    impl CustomBinaryFormat {
        pub fn new(magic: u32, version: u16) -> Self {
            Self {
                magic_number: magic,
                version,
                flags: 0,
            }
        }

        /// 写入文件头
        pub fn write_header(&self, writer: &mut BinaryWriter) -> EngineResult<()> {
            writer.write_u32_le(self.magic_number)?;
            writer.write_u32_le(self.version as u32)?;
            writer.write_u32_le(self.flags as u32)?;
            Ok(())
        }

        /// 读取文件头
        pub fn read_header(&mut self, reader: &mut BinaryReader) -> EngineResult<()> {
            let magic = reader.read_u32_le()?;
            if magic != self.magic_number {
                return Err(anyhow::anyhow!("Invalid magic number: expected {}, got {}", self.magic_number, magic));
            }

            let version = reader.read_u32_le()? as u16;
            if version > self.version {
                return Err(anyhow::anyhow!("Unsupported version: {}", version));
            }

            self.flags = reader.read_u32_le()? as u16;
            Ok(())
        }

        /// 序列化数据
        pub fn serialize_data<T: Serialize>(&self, data: &T) -> EngineResult<Vec<u8>> {
            let mut writer = BinaryWriter::new();
            
            // 写入头部
            self.write_header(&mut writer)?;
            
            // 序列化数据
            let serialized = to_compact_binary(data)?;
            writer.write_u32_le(serialized.len() as u32)?;
            writer.write_bytes(&serialized)?;
            
            Ok(writer.into_bytes())
        }

        /// 反序列化数据
        pub fn deserialize_data<T: for<'de> Deserialize<'de>>(&mut self, data: &[u8]) -> EngineResult<T> {
            let mut reader = BinaryReader::new(data.to_vec());
            
            // 读取头部
            self.read_header(&mut reader)?;
            
            // 读取数据长度
            let data_len = reader.read_u32_le()? as usize;
            let data_bytes = reader.read_bytes(data_len)?;
            
            // 反序列化数据
            let result = from_compact_binary(&data_bytes)?;
            Ok(result)
        }
    }

    /// 计算二进制数据的CRC32校验和
    pub fn calculate_crc32(data: &[u8]) -> u32 {
        let mut crc = crc32fast::Hasher::new();
        crc.update(data);
        crc.finalize()
    }

    /// 验证CRC32校验和
    pub fn verify_crc32(data: &[u8], expected: u32) -> bool {
        calculate_crc32(data) == expected
    }

    /// 添加CRC32校验和到数据
    pub fn add_checksum(data: &[u8]) -> Vec<u8> {
        let mut result = data.to_vec();
        let checksum = calculate_crc32(data);
        result.extend_from_slice(&checksum.to_le_bytes());
        result
    }

    /// 验证并移除CRC32校验和
    pub fn verify_and_remove_checksum(data: &[u8]) -> EngineResult<Vec<u8>> {
        if data.len() < 4 {
            return Err(anyhow::anyhow!("Data too short for checksum"));
        }

        let (payload, checksum_bytes) = data.split_at(data.len() - 4);
        let expected_checksum = u32::from_le_bytes([
            checksum_bytes[0],
            checksum_bytes[1],
            checksum_bytes[2],
            checksum_bytes[3],
        ]);

        if verify_crc32(payload, expected_checksum) {
            Ok(payload.to_vec())
        } else {
            Err(anyhow::anyhow!("Checksum verification failed"))
        }
    }
}
