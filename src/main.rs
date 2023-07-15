pub mod protocol;

use file_diff::diff_files;

use protocol::guaranteed_udp::GuaranteedUdp;
use protocol::tcp::Tcp;
use protocol::udp::Udp;
use std::fs::File;
use std::io::{Read, Result, Write};
use std::net::SocketAddr;
use tokio::net::{TcpStream, UdpSocket};

#[tokio::main]
async fn main() -> Result<()> {
  // Read the file from the first position of the cli param
  let args: Vec<String> = std::env::args().collect();
  let execution_mode = args.get(1).expect("Please provide an execution mode");

  match execution_mode.as_str() {
    "sender" => sender_routine().await.unwrap(),
    "receiver" => receiver_routine().await.unwrap(),
    _ => panic!("Please provide a valid execution mode (sender or receiver)"),
  }

  Ok(())
}

async fn sender_routine() -> Result<()> {
  // Read the file from the first position of the cli param
  let args: Vec<String> = std::env::args().collect();
  let file_name = args.get(2).expect("Please provide a file to send");
  let protocol = args.get(3).expect("Please provide a protocol to use");
  let packet_size = args
    .get(4)
    .expect("Please provide a packet size to use")
    .parse()
    .unwrap();

  let mut file = File::open(file_name)?;
  let mut file_data = Vec::new();
  file.read_to_end(&mut file_data)?;

  // Receiver address
  let receiver_address: SocketAddr = "127.0.0.1:2000".parse().unwrap();

  match protocol.as_str() {
    "udp" => {
      // Sender UDP socket
      let udp_socket = UdpSocket::bind("127.0.0.1:2001").await?;

      Udp::send_file(file_data, &receiver_address, packet_size, &udp_socket).await?;
    }
    "gudp" => {
      // Sender UDP socket
      let udp_socket = UdpSocket::bind("127.0.0.1:2001").await?;

      GuaranteedUdp::send_file(file_data, &receiver_address, packet_size, &udp_socket).await?;
    }
    "tcp" => {
      let stream = TcpStream::connect(receiver_address).await?;

      Tcp::send_file(file_data, packet_size, stream).await?;
    }
    _ => {
      panic!("Unknown protocol: {}", protocol);
    }
  }

  println!("File sent successfully.");
  Ok(())
}

async fn receiver_routine() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  let protocol = args.get(2).expect("No protocol provided");
  let packet_size: usize = args.get(3).unwrap_or(&String::from("100")).parse().unwrap();

  let reference_file = "assets/100mb.txt";

  match protocol.as_str() {
    "udp" => {
      let udp_socket = UdpSocket::bind("127.0.0.1:2000").await.unwrap();
      let file_data = Udp::receive_file(&udp_socket, packet_size).await.unwrap();
      write_file(&file_data, "udp.txt");
      if !verify_files(reference_file, "udp.txt") {
        panic!("Files are not equal");
      }
    }
    "gudp" => {
      let udp_socket = UdpSocket::bind("127.0.0.1:2000").await.unwrap();
      let file_data = GuaranteedUdp::receive_file(&udp_socket, packet_size)
        .await
        .unwrap();
      write_file(&file_data, "gudp.txt");
      if !verify_files(reference_file, "gudp.txt") {
        panic!("Files are not equal");
      }
    }
    "tcp" => {
      let file_data = Tcp::receive_file(packet_size, "127.0.0.1:2000")
        .await
        .unwrap();
      write_file(&file_data, "tcp.txt");
      if !verify_files(reference_file, "tcp.txt") {
        panic!("Files are not equal");
      }
    }
    _ => {
      panic!("Invalid protocol provided");
    }
  }

  println!("File received and written successfully.");
  Ok(())
}

fn verify_files(file_name1: &str, file_name2: &str) -> bool {
  let mut file1 = File::open(file_name1).unwrap();
  let mut file2 = File::open(file_name2).unwrap();

  diff_files(&mut file1, &mut file2)
}

fn write_file(file_data: &[u8], file_name: &str) {
  println!("Received {} bytes", file_data.len());
  let mut file = File::create(file_name).unwrap();
  file.write_all(file_data).unwrap();
}
