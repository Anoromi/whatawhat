use std::ptr::null;

use anyhow::Result;
use xcb::{
    screensaver::{QueryInfo, QueryInfoReply},
    x::Window,
    Xid,
};
// use x11::xlib::XOpenDisplay;

use super::ActiveWindowData;

pub fn get_active() -> Result<ActiveWindowData> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = conn.get_setup();
    let k = setup.roots().next().unwrap();
    let wnd = k.root();
    println!("{:?}", k.root());
    
    let hehe = conn.send_request(&QueryInfo {
        drawable: xcb::x::Drawable::Window(wnd),
    });
    let reply: QueryInfoReply = conn.wait_for_reply(hehe)?;
    dbg!(reply.ms_since_user_input());
    
    todo!()
    
}

pub fn is_afk() -> Result<()> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = conn.get_setup();
    // setup.roots().collect()

    let hehe = conn.send_request(&QueryInfo {
        drawable: xcb::x::Drawable::Window(Window::none()),
    });
    let reply: QueryInfoReply = conn.wait_for_reply(hehe)?;
    reply.ms_since_user_input();
    // reply

    // let display = unsafe { XOpenDisplay(null()) };

    todo!()
}
