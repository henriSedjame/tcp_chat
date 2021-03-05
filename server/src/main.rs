use std::io::{ErrorKind, Read, Write, Error};
use std::net::TcpListener;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

const LOCAL_IP_ADR: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;



fn main() {

    // Create server bind to address
    let server = TcpListener::bind(LOCAL_IP_ADR).expect("Listener failed to bind");

    //Set server to non-blocking mode
    server.set_nonblocking(true).expect("Failed to initialize non-blocking");

    // Initialize vec of clients
    let mut clients = vec![];

    // Create a channel with transmitter tx and receiver rx of String typed message
    let (tx, rx) = mpsc::channel::<String>();

    loop {

        // If server connect successfully
        if let Ok((mut tcp_stream, socket_adr)) = server.accept() {
            println!("Client {} connected", socket_adr);

            //Clone transmitter
            let tx = tx.clone();

            //Clone tcp_stream as client
            let client = tcp_stream.try_clone().expect("Failed to clone client");

            //add client to clients vertor
            clients.push(client);

            // Create a new thread
            thread::spawn(move || loop {

                // Initialize buffer with size of MSG_SIZE and with initial value 0
                let mut buff = vec![0; MSG_SIZE];

                // Read from client
                match tcp_stream.read_exact(&mut buff) {
                    Ok(_) => {
                        // Retrieve client message
                        let msg_buff = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                        let msg = String::from_utf8(msg_buff).expect("Invalid utf8 message");

                        println!("{} : {:?}", socket_adr, msg);

                        // Send message to receiver
                        tx.send(msg).expect("Failed to send message to receiver");
                    }
                    Err(ref err)  if err.kind() == ErrorKind::WouldBlock => (),
                    Err(_) => {
                        println!("Closing connection with : {}", socket_adr);
                        break;
                    }
                }

                //Sleep thread
                sleep();
            });
        }


        // If receiver receive message successfully
        if let Ok(receivedMsg) = rx.try_recv() {
            clients = clients
                .into_iter()
                .filter_map(|mut client| {
                    let mut buff = receivedMsg.clone().into_bytes();
                    buff.resize(MSG_SIZE, 0);
                    client.write_all(&buff).map(|_| client).ok()
                })
                .collect::<Vec<_>>();
        }

        sleep();
    }
}

fn sleep() {
    thread::sleep(Duration::from_millis(100));
}
