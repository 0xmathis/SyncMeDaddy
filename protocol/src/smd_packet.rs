use anyhow::Result;
use log::trace;
use std::fmt;
use std::io::{Read, Write};
use std::net::TcpStream;

use crate::smd_type::SMDtype;


pub struct SMDpacket {
    version: u8,
    data_type: SMDtype,
    data_length: u32,
    data: Vec<u8>,
}

impl SMDpacket {
    pub fn new(version: u8, data_type: SMDtype, data: Vec<u8>) -> Self {
        Self {
            version,
            data_type,
            data_length: data.len() as u32,
            data,
        }
    }

    pub fn send_to(self, mut stream: &TcpStream) -> Result<()> {
        stream.write_all(&[self.version])?;
        stream.write_all(&[self.data_type.to_value()])?;
        stream.write_all(&self.data_length.to_be_bytes())?;
        stream.write_all(&self.data)?;

        trace!("To: {:?} | Sending: {}", stream.peer_addr(), self);
        Ok(())
    }

    pub fn receive_from(mut stream: &TcpStream) -> Result<SMDpacket> {
        let mut version: [u8; 1] = [0; 1];
        let mut data_type: [u8; 1] = [0; 1];
        let mut data_length: [u8; 4] = [0; 4];

        stream.read_exact(&mut version)?;
        stream.read_exact(&mut data_type)?;
        stream.read_exact(&mut data_length)?;

        let version: u8 = version[0];
        let data_type: SMDtype = SMDtype::from_value(data_type[0]);
        let data_length: u32 = u32::from_be_bytes(data_length);
        let mut data: Vec<u8> = vec![];
        data.resize(data_length as usize, 0);

        stream.read_exact(&mut data)?;
        let packet: Self = Self::new(version, data_type, data);

        trace!("From: {:?} | Received: {}", stream.peer_addr(), packet);
        Ok(packet)
    }

    pub fn get_data(&self) -> &Vec<u8> {
        &self.data
    }

    pub fn get_type(&self) -> &SMDtype {
        &self.data_type
    }
}

impl fmt::Display for SMDpacket {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{ version: {} | type: {:?} | length: {} }}", self.version, self.data_type, self.data_length)
    }
}
