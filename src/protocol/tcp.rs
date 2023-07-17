use std::io::{Read, Result};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

use std::net::TcpListener;

pub struct Tcp {}

impl Tcp {
  pub async fn send_file(
    file_data: Vec<u8>,
    packet_size: usize,
    mut stream: TcpStream,
  ) -> Result<()> {
    let packets = file_data.chunks(packet_size).enumerate();

    // println!("Sending {} packets of size {}", packets.len(), packet_size);

    // Send each packet to the receiver
    for (_packet_num, packet) in packets {
      let _send = stream.write(packet).await?;
      stream.flush().await?;
    }

    Ok(())
  }

  pub async fn receive_file(packet_size: usize, receiver_address: &str) -> Result<Vec<u8>> {
    let mut received_data = Vec::new();
    let mut buffer = vec![0u8; packet_size];

    let (mut stream, _addr) = TcpListener::bind(receiver_address)
      .unwrap()
      .accept()
      .unwrap();

    loop {
      let number_of_bytes = stream.read(&mut buffer)?;
      if number_of_bytes == 0 {
        break;
      }
      received_data.extend_from_slice(&buffer[..number_of_bytes]);
    }

    Ok(received_data)
  }
}
