use std::io::{self, ErrorKind, Read, Write, Error};
use std::net::TcpStream;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use std::sync::mpsc::TryRecvError;

const LOCAL_IP_ADR: &str = "127.0.0.1:6000";
const MSG_SIZE: usize = 32;


fn main() {

    let mut client = TcpStream::connect(LOCAL_IP_ADR).expect("Strean failed to connect");

    client.set_nonblocking(true).expect("Failed to set nonblocking mode");

    let (tx, rx) = mpsc::channel::<String>();

    thread::spawn(move|| loop{
        let mut buff = vec![0; MSG_SIZE];

        match client.read_exact(&mut buff){
            Ok(_) => {
                let msg_buff = buff.into_iter().take_while(|&x| x != 0).collect::<Vec<_>>();
                println!("Message received : {:?}", msg_buff);
            }
            Err(ref err) if err.kind() == ErrorKind::WouldBlock => (),
            Err(_) => {
                println!("Connection with server severed");
                break;
            }
        };

        match rx.try_recv() {
            Ok(receivedMsg) => {
                let mut buff = receivedMsg.clone().into_bytes();
                buff.resize(MSG_SIZE, 0);
                client.write_all(&buff).expect("Writing socket failed");

                println!("Message sent : {:?}", receivedMsg);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => break
        }

        thread::sleep(Duration::from_millis(100));
    });

    println!("Write a message: ");

    loop {
        let mut buff = String::new();
        io::stdin().read_line(&mut buff).expect("Read from stdin failed");
        let msg = buff.trim().to_string();

        if msg == ":quit" || tx.send(msg).is_err() {break}
    }

    println!("Bye");

}
