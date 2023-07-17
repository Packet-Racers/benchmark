pub mod protocol;

use file_diff::diff_files;

use protocol::guaranteed_udp::GuaranteedUdp;
use protocol::tcp::Tcp;
use protocol::udp::Udp;
use std::fs::File;
use std::io::{Read, Result, Write};
use std::net::SocketAddr;
use std::thread;
use std::time::{Duration, Instant};
use tokio::net::{TcpStream, UdpSocket};

const EXECUTION_TIMES: usize = 10;

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
  let receiver_address: SocketAddr = "192.168.0.128:2000".parse().unwrap();
  let bind_addr = "0.0.0.0:2001";

  for i in 0..EXECUTION_TIMES {
    let execution_time = match protocol.as_str() {
      "udp" => {
        // Sender UDP socket
        let udp_socket = UdpSocket::bind(bind_addr).await?;

        let start_time = Instant::now();
        Udp::send_file(
          file_data.clone(),
          &receiver_address,
          packet_size,
          &udp_socket,
        )
        .await?;
        let end_time = Instant::now();
        end_time - start_time
      }
      "gudp" => {
        // Sender UDP socket
        let udp_socket = UdpSocket::bind(bind_addr).await?;

        let start_time = Instant::now();
        let total_retries = GuaranteedUdp::send_file(
          file_data.clone(),
          &receiver_address,
          packet_size,
          &udp_socket,
        )
        .await?;
        if total_retries > 0 {
          println!("Total retries: {}", total_retries);
        }
        let end_time = Instant::now();
        end_time - start_time
      }
      "tcp" => {
        let stream = TcpStream::connect(receiver_address).await?;

        let start_time = Instant::now();
        Tcp::send_file(file_data.clone(), packet_size, stream).await?;
        let end_time = Instant::now();
        end_time - start_time
      }
      _ => {
        panic!("Unknown protocol: {}", protocol);
      }
    };

    println!("{}: {}", i, execution_time.as_millis());
    thread::sleep(Duration::from_millis(1000));
  }

  println!("File sent successfully.");
  Ok(())
}

async fn receiver_routine() -> Result<()> {
  let args: Vec<String> = std::env::args().collect();
  let protocol = args.get(2).expect("No protocol provided");
  let packet_size: usize = args.get(3).unwrap_or(&String::from("100")).parse().unwrap();

  let bind_addr = "0.0.0.0:2000";

  let reference_file = "assets/100mb.txt";

  for i in 0..EXECUTION_TIMES {
    let is_files_equals = match protocol.as_str() {
      "udp" => {
        let udp_socket = UdpSocket::bind(bind_addr).await.unwrap();
        let file_data = Udp::receive_file(&udp_socket, packet_size).await.unwrap();
        write_file(&file_data, "udp.txt");
        verify_files(reference_file, "udp.txt")
      }
      "gudp" => {
        let udp_socket = UdpSocket::bind(bind_addr).await.unwrap();
        let file_data = GuaranteedUdp::receive_file(&udp_socket, packet_size)
          .await
          .unwrap();
        write_file(&file_data, "gudp.txt");
        verify_files(reference_file, "gudp.txt")
      }
      "tcp" => {
        let file_data = Tcp::receive_file(packet_size, bind_addr).await.unwrap();
        write_file(&file_data, "tcp.txt");
        verify_files(reference_file, "tcp.txt")
      }
      _ => {
        panic!("Invalid protocol provided");
      }
    };

    let files_equals_text = if is_files_equals {
      String::from("Files are equals")
    } else {
      String::from("Files are NOT equals")
    };

    println!("{}: {}", i, files_equals_text);
  }

  // println!("File received and written successfully.");
  Ok(())
}

fn verify_files(file_name1: &str, file_name2: &str) -> bool {
  let mut file1 = File::open(file_name1).unwrap();
  let mut file2 = File::open(file_name2).unwrap();

  diff_files(&mut file1, &mut file2)
}

fn write_file(file_data: &[u8], file_name: &str) {
  let mut file = File::create(file_name).unwrap();
  file.write_all(file_data).unwrap();
}
