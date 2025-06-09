# MoQ
__MQTT over QUIC__

## Overview
This is the crude initial start at writing the MQTT protocol for use over QUIC rather than TCP. The primary motivation for this was as part of Drexel CCI's CS544-Computer Networks Protocol Asssignment.

The starter code of the QUIC client and server were provided provided from [the class Githib page,](https://github.com/ArchitectingSoftware/CS544-Class-Demo-Files/tree/main/quic) in a variety of languages. There is also a cert generator there, but I had my best luck using the one from [the AWS s2n-quic Github page.](https://github.com/aws/s2n-quic/tree/main/examples/s2n-mtls/certs)

## Demo Program
The starter code was set up to run out of the command line. It was a basic client/server implementation that would echo back what the client sent. MQTT is a little different, having the Broker sit between a Subscriber and Publisher. To see this in action, you need to start three different terminals.

1. **Broker**  
   AKA, the server, should be running first. If not, the clinent(s) will just sit and wait.   
   ```sh
   cargo run -- -a "127.0.0.1" -p "4433" -c "./certs/server-cert.pem" -k "./certs/server-key.pem" server
   ```
   It could also be as simple as...
   ```sh
   cargo run server
   ```
   if you have your certs in the right directory and the network defaults work on your machine. I had to do some playing around to get started, and then had the line handy in my terminal.

2. **Subscriber**
   It works best to start a subscriber next since after it subscribes to a topic it'll just sit there and wait for publications. It is a little bit of an infinite loop program so you have to ctrl+c to get out.
   ```sh
   cargo run -- -a "127.0.0.1" -p "4433" -c "./certs/ca-cert.pem" -t "Test" subscriber
   ```
   The -t arguement is the topic name. It must match exactly for both Publisher and Subscriber, i.e. Test is not the same as test.

3. **Publisher**
   The last step is start a Publisher.
   ```sh
   cargo run -- -a "127.0.0.1" -p "4433" -c "./certs/ca-cert.pem" -t "Test" -m "The quick brown fox jumps over the lazy dog"  publisher
   ```
   As mentioned above the -t arguement is critial to match. The -m arguement is what the publisher will publish. The publisher is not a fancy application in the sense that it publishes and shuts down. If I hadn't sunk so much time into tweaking the server to broadcast messages based on matching topic names, I would've wrote a fancier client that stays open and publishes data from stdin instead. Was working with just marginally better than pub/sub Hello, world!


