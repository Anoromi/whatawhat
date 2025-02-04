use std::time::Duration;

use anyhow::Result;
use interprocess::local_socket::{
    traits::tokio::Listener, GenericNamespaced, ListenerOptions, ToNsName,
};
use tokio::{io::{AsyncBufReadExt, BufReader}, time::sleep};

use crate::daemon::update::DAEMON_CHANNEL_NAME;


pub async fn receive_interprocess_messages() -> Result<()> {
    let name = DAEMON_CHANNEL_NAME.to_ns_name::<GenericNamespaced>()?;
    let listener = ListenerOptions::new().name(name).create_tokio()?;
    let mut buffer = String::with_capacity(1024);

    println!("hehe hehe hehe");
    for conn in listener.accept().await {
        // Wrap the connection into a buffered receiver right away
        // so that we could receive a single line from it.
        let mut conn = BufReader::new(conn);
        println!("Incoming connection!");

        // Since our client example sends first, the server should receive a line and only then
        // send a response. Otherwise, because receiving from and sending to a connection cannot
        // be simultaneous without threads or async, we can deadlock the two processes by having
        // both sides wait for the send buffer to be emptied by the other.
        conn.read_line(&mut buffer).await?;
        // let value = serde_json::from_str(&buffer)?;
        // // Now that the receive has come through and the client is waiting on the server's send, do
        // // it. (`.get_mut()` is to get the sender, `BufReader` doesn't implement a pass-through
        // // `Write`.)
        // conn.get_mut().write_all(b"Hello from server!\n")?;

        // Print out the result, getting the newline for free!
        print!("Client answered: {buffer}");

        // Clear the buffer so that the next iteration will display new data instead of messages
        // stacking on top of one another.
        buffer.clear();
    }

    // let console_window = unsafe { GetConsoleWindow() };
    // unsafe {
    //     // let k = dbg!(ShowWindow(console_window, SW_HIDE));
    //     // let t: isize = 0x00000008;
    //     // dbg!(SetWindowLongPtrA(
    //     //     console_window,
    //     //     GWL_EXSTYLE,
    //     //     t // transmute::<_, i32>(WS_EX_NOACTIVATE.0).into()
    //     // ));
    //     // sleep(Duration::from_secs(10));
    //     // k.unwrap();
    // };
    println!("Done");

    // unsafe { AllocConsole().unwrap() }
    loop {
        // unsafe { FreeConsole().unwrap() };
        println!("Hello there");
        sleep(Duration::from_millis(1000)).await;
    }
}
