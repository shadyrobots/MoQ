use color_eyre::eyre::Result;
use s2n_quic::{client::Connect, Client};
use tokio::time::{sleep, timeout};
use std::time::Duration;
use std::{path::Path, net::SocketAddr};
use std::net::ToSocketAddrs;
use tokio::sync::Mutex;
use std::sync::Arc;

use crate::protocol::mqtt::{MQTTProtocol, PacketType};

#[derive(Debug)]
struct ClientOptions{
  address: String,
  port: u16,
  cert: String,
  topic: String,
  client_type: String,
  message: String,
}

#[tokio::main]
async fn run(options:ClientOptions) -> Result<()> {
    let host_port_string = format!("{}:{}", 
      options.address, options.port).to_socket_addrs()?.next().unwrap();

    let addr: SocketAddr = "0.0.0.0:0".parse()?;
    let client = Client::builder()
        .with_tls(Path::new(&options.cert))?
        .with_io(addr)?
        .start()?;
    
    println!("Connecting {}...", options.client_type);
    let connect = Connect::new(host_port_string).with_server_name("localhost");
    let mut connection = client.connect(connect).await?;
    
    println!("{} connected...", options.client_type);
    // ensure the connection doesn't time out with inactivity
    connection.keep_alive(true)?;

    // open a new stream and split the receiving and sending sides
    let stream = connection.open_bidirectional_stream().await?;
    let (receive_stream, mut send_stream) = stream.split();
    // receive_stream shared mutable access if needed across tasks
    let receive_stream_arc = Arc::new(Mutex::new(receive_stream));

    let topic= &options.topic;

    if options.client_type.to_lowercase() == "publisher" {
      //let mut i = 0;
      
      //while i < 10{
        let payload: String = options.message;
        let payload_length = payload.len() as u32;
        let msg = MQTTProtocol::new(payload, PacketType::PUBLISH, 0, topic.to_string(), 0, payload_length);
        //println!("<== TO SERVER ==\n {} \n===", msg.to_json().unwrap());  
        msg.print_debug_msg("TO SERVER");
        let data = msg.to_bytes().unwrap();
        send_stream.send(data.into()).await.expect("stream should be open");

        sleep(Duration::new(10,0)).await;
      //  i += 1;
      //}
    }

    if options.client_type.to_lowercase() == "subscriber" {
      //Subscribe to a topic on the server
      let payload: String = "".to_string();
      let payload_length = payload.len() as u32;
      let msg = MQTTProtocol::new(payload, PacketType::SUBSCRIBE, 0, topic.to_string(), 0, payload_length);
      //println!("<== TO SERVER ==\n {} \n===", msg.to_json().unwrap());  
      msg.print_debug_msg("TO SERVER");
      let data = msg.to_bytes().unwrap();
      send_stream.send(data.into()).await.expect("stream should be open");
    
      println!("Subscriber listening for messages on topic: {}", msg.topic);
      loop{
        let received_data = timeout(Duration::from_secs(10), receive_stream_arc.lock().await.receive()).await;
        
        match received_data {
          Ok(Ok(Some(rdata))) => {
            let msg = MQTTProtocol::from_bytes(rdata.to_vec())?;
            if msg.decode_packet_type().unwrap() == 3 {
              msg.print_debug_msg("FROM PUBLISHER");
            } else {
              msg.print_debug_msg("UNKNOWN MESSAGE");
            }
          }
        Ok(Ok(None)) => {
          println!("Stream closed by broker, exiting");
          break;
        } 
        Ok(Err(e))=>{
          println!("Error occured receiving from stream: {}", e);
          break;
        }
        #[allow(dead_code)]
        Err(_e) => {
          //println!("Timeout error after 10s, will keep listening");
        }
      }
    }
    }
    Ok(())
}


pub fn do_client(address: String, port: u16, cert: String, topic: String, client_type: String, message: String) -> Result<()> {
  println!("Starting client...");
  println!("Connecting to {address} on port {port}...");

  let options = ClientOptions {
    address,
    port,
    cert,
    topic,
    client_type,
    message,
  };

  println!("{}, {}", options.client_type, options.topic);
  run(options)?;

  Ok(())
}
