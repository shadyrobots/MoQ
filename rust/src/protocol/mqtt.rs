use bincode::{deserialize, serialize};
use color_eyre::eyre::{eyre, Result, WrapErr};
use serde::{Deserialize, Serialize};

#[repr(u8)]
#[allow(dead_code)]
pub enum PacketType {
  CONNECT = 1,
  CONNACK = 2,
  PUBLISH = 3,
  PUBACK = 4,
  PUBREC = 5,
  PUBREL = 6,
  PUBCOM = 7,
  SUBSCRIBE = 8,
  SUBACK = 9,
  UNSUBSCRIBE = 10,
  UNSUBACK = 11,
  PINGREQ = 12,
  PINGRESP = 13,
  DISCONNECT = 14,
}


#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MQTTProtocol {
  pub command_header: u8,
  pub remaining_length: u32,
  pub variable_header: u128,
  pub topic: String,
  pub payload: String,
  //ver: u8,
  //pub mtype: u8,
  //pub msg: String,
}

#[allow(dead_code)]
impl MQTTProtocol {
  pub fn new(payload: String, packet_type: PacketType, flags: u8, topic: String, variable_header_len: u32, payload_len: u32) -> Self {
    let command_header: u8 = ((packet_type as u8) << 4) + (flags & 0b00001111);
    let remaining_length : u32 = variable_header_len + payload_len;
    let variable_header: u128 = 0;
    MQTTProtocol { command_header, remaining_length, topic, variable_header, payload}
    //EchoProtocol { ver:PROTO_VERSION, mtype, msg, command_header, remaining_length}
  }

  pub fn decode_packet_type(&self) -> Result<u8> {
    Ok(self.command_header >> 4)
  }

  pub fn from_json(raw: &str) -> Result<Self> {
    let message = serde_json::from_str(raw)?;

    Ok(message)
  }

  pub fn from_bytes(raw: Vec<u8>) -> Result<Self> {
    let decoded_data: MQTTProtocol = deserialize(&raw).unwrap();
    //println!("{:?}",serde_json::to_string(&decoded_data));
    //let raw_json =
    //b  String::from_utf8(decoded_data.to_bytes()?).wrap_err_with(|| eyre!("Unable to parse bytes as UTF8 string"))?;
    //println!("{}", raw_json);

    Ok(decoded_data)
  }

  pub fn to_bytes(&self) -> Result<Vec<u8>> {
    // Implement binary bits here
    //Ok(self.to_json()?.as_bytes().to_owned())
    //Ok(self.to_json()?.into_bytes())
    Ok(serialize(&self).unwrap())
  }

  pub fn to_json(&self) -> Result<String> {
    let pretty_json = serde_json::to_string_pretty(self)
      .wrap_err_with(|| eyre!("Problem serializing Message to JSON"))?;

    Ok(pretty_json)
  }
  // Define a method for future use
  #[allow(dead_code)]
  pub fn print_debug_msg(&self, msg: &str) {
    println!("{}\nHeader: '{:#010b}'\n{}", msg, self.command_header, self.payload)//  .to_json().unwrap());
  }

  // Define a method for future use
  #[allow(dead_code)]
  pub fn to_string(&self) -> String {
    format!("{:#?}", self)
  }
}



#[cfg(test)]
mod tests {
  use super::*;

  use pretty_assertions::assert_eq;

  #[test]
  fn constructor_sanity() {
    let message = MQTTProtocol::new( 
      "Hello, world!".to_string(), PacketType::PUBLISH, 0, "Test_Topic".to_string(), 0, 0);

    assert_eq!(message.remaining_length, 1);
  }
}
