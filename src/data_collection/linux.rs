use std::ptr::null;

use anyhow::Result;
use xcb::{
    screensaver::{QueryInfo, QueryInfoReply},
    x::{GetProperty, InternAtom, InternAtomCookie, Window, ATOM_WINDOW},
    Xid,
};
// use x11::xlib::XOpenDisplay;

use super::ActiveWindowData;

// pub fn get_active() -> Result<ActiveWindowData> {
//     let (conn, screen_num) = xcb::Connection::connect(None)?;
//     let setup = conn.get_setup();
//     let k = setup.roots().next().unwrap();
//     let wnd = k.root();
//     println!("{:?}", k.root());
//
//     let hehe = conn.send_request(&QueryInfo {
//         drawable: xcb::x::Drawable::Window(wnd),
//     });
//     let reply: QueryInfoReply = conn.wait_for_reply(hehe)?;
//     dbg!(reply.ms_since_user_input());
//
//     todo!()
//
// }

pub fn get_active() -> Result<ActiveWindowData> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let setup = conn.get_setup();
    let k = setup.roots().collect::<Vec<_>>();
    
    dbg!(&k.len());
    let root = k[0].root();
    let active_window_cookie = conn.send_request(&InternAtom {
        only_if_exists: false,
        name: b"_NET_ACTIVE_WINDOW",
    });
    let active_window_reply = conn.wait_for_reply(active_window_cookie)?.atom();
    let property_cookie = conn.send_request(&GetProperty {
        delete: false,
        window: root,
        property: active_window_reply,
        r#type: ATOM_WINDOW,
        long_offset: 0,
        long_length: 1,
    });
    let property = conn.wait_for_reply(property_cookie)?;
    dbg!(property);
    // println!("{:?}", k.root());

    let hehe = conn.send_request(&QueryInfo {
        drawable: xcb::x::Drawable::Window(root),
    });
    let reply: QueryInfoReply = conn.wait_for_reply(hehe)?;
    dbg!(reply.ms_since_user_input());

    Ok(ActiveWindowData {
        title: "".into(),
        process_name: "".into(),
    })
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
