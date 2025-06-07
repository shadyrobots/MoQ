use color_eyre::eyre::Result;
use s2n_quic::{Server, Connection};
use std::{path::Path, sync::Arc};
use std::net::ToSocketAddrs;
use tokio::sync::{broadcast, RwLock, Mutex}; // Added Mutex for stream
use std::collections::HashMap;

use crate::protocol::mqtt::{MQTTProtocol};

#[derive(Debug)]
struct ServerOptions{
  address: String,
  port: u16,
  cert: String,
  key: String,
}

type TopicMap = Arc<RwLock<HashMap<String, broadcast::Sender<MQTTProtocol>>>>;

#[tokio::main]
async fn run(options: ServerOptions) -> Result<()>  {

  let host_port_string = format!("{}:{}", 
    options.address, options.port).to_socket_addrs()?.next().unwrap();
  let mut server = Server::builder()
        .with_tls((Path::new(&options.cert), Path::new(&options.key)))?
        .with_io(host_port_string)?
        .start()?;
  println!("Broker listening on {}", host_port_string);

  // shared topic map for all connections
  let topics: TopicMap = Arc::new(RwLock::new(HashMap::new()));

  while let Some(connection) = server.accept().await {
    let remote_addr = connection.remote_addr();
    println!("New connection from: {:?}", remote_addr);

    let topics_clone = Arc::clone(&topics);

    // Spawn a new asynchronous task to handle each incoming QUIC connection
    tokio::spawn(async move {
      if let Err(e) = handle_connection(connection, topics_clone).await {
        println!("Error handling connection from {:?}: {}", remote_addr, e);
      }
      println!("Connection from {:?} closed.", remote_addr);
    });
  }
  Ok(())
}

async fn handle_connection(mut connection: Connection, topics: TopicMap,) -> Result<()> {
  while let Ok(Some(stream)) = connection.accept_bidirectional_stream().await {
    let stream_id = stream.id();
    
    let (receive_stream, send_stream) = stream.split();

    let send_stream_arc = Arc::new(Mutex::new(send_stream));
    let receive_stream_arc = Arc::new(Mutex::new(receive_stream));

    let topics_clone_for_inbound = Arc::clone(&topics); // Clone for the stream processing task
    //let topics_clone_for_outbound = Arc::clone(&topics);

    tokio::spawn({
      //let send_stream_arc_for_acks = Arc::clone(&send_stream_arc);
      let receivce_stream_arc_for_inbound = Arc::clone(&receive_stream_arc);
      async move {
      // Loop to continuously receive data from the client on this stream
        while let Ok(Some(data)) = receivce_stream_arc_for_inbound.lock().await.receive().await {
          let packet = match MQTTProtocol::from_bytes(data.to_vec()) {
            Ok(p) => p,
            Err(e) => {
              println!("Failed to parse MQTT packet with error: {}", e);
              continue;
            }
          };
          packet.print_debug_msg("FROM CLIENT");
          println!("{}", packet.decode_packet_type().unwrap());
          match packet.decode_packet_type().unwrap() {
            3 => { //PUBLSIH
              //println!("Received PUBLISH message on stream {}", stream_id);
              let topic_name = packet.topic.clone();
              let payload_to_broadcast = packet.clone();

              let topics_guard = topics_clone_for_inbound.read().await; 
              if let Some(sender) = topics_guard.get(&topic_name) {
                let num_receivers  = sender.send(payload_to_broadcast.clone()); 
                println!("Broadcasted PUBLISH message to topic: '{}' to {:?} subscribers", topic_name, num_receivers);
              } else {
                println!("No subscribers for topic: '{}'", topic_name);
              }

              //TODO implement QoS and PUBACK here
            }
            8 => { //SUBSCRIBE
              let topic_name = packet.topic.clone();
              let mut topics_guard = topics_clone_for_inbound.write().await;
              
              let sender = topics_guard
                .entry(topic_name.clone())
                .or_insert_with(|| {
                // If topic doesn't exist, create a new broadcast channel
                let (tx, _) = broadcast::channel(100); // 100-message buffer
                println!("Created new broadcast channel for topic: '{}'", topic_name);
                tx
              });

              let receiver = sender.subscribe();
              println!("Client subscribed to topic: '{}' on stream {}", topic_name, stream_id);

              //TODO implement SUBACK here

              tokio::spawn({
                let stream_arc_for_outbound = Arc::clone(&send_stream_arc);
                let topic_name_for_log = topic_name.clone();
                async move {
                  let mut sub_rx = receiver;
                  loop {
                    match sub_rx.recv().await {
                        Ok(msg_to_forward) => {
                          //println!("[Forwarder for '{}', Stream {}]: Received message from broadcast channel.", topic_name_for_log, stream_id);
                          let payload = msg_to_forward.to_bytes().unwrap();

                          let mut stream_guard = stream_arc_for_outbound.lock().await;
                          //println!("locked steam for sending");
                          match stream_guard.send(payload.into()).await { 
                            Ok(_) => {
                              msg_to_forward.print_debug_msg(&format!("FORWARDING TO SUBSCRIBER (Topic: '{}', Stream: {})", topic_name_for_log, stream_id));
                            },
                            Err(e) => {
                              eprintln!("[Forwarder for '{}', Stream {}]: Error sending to stream: {}. Breaking forwarder loop.", topic_name_for_log, stream_id, e);
                              break; // Fatal error sending, break loop
                            }
                          }
                        }
                        Err(e) => {
                          println!("Subscriber stream {} recieved error: {}", stream_id, e);
                          break;
                        }
                    }
                  }
                }
              });
            }
            _ => { //More features here some day
              println!("Recieved packet for unimplemented feature: {}", packet.decode_packet_type().unwrap());
            }
          }
        }
      }
    });
  }
  Ok(())
}
 

pub fn do_server(address: String, port: u16, cert:String, key:String) -> Result<()> {
  println!("Starting MoQ Broker...");

  let options = ServerOptions {
    address,
    port,
    cert,
    key,
  };

  run(options)?;

  // return Ok if there is no error
  Ok(())
}
