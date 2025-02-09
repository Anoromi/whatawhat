use std::ptr::null;

use anyhow::Result;
use xcb::{
    screensaver::{QueryInfo, QueryInfoReply},
    x::{
        GetInputFocus, GetProperty, GrabServer, InputFocus, InternAtom, InternAtomCookie, QueryTree, UngrabServer, Window, ATOM_WINDOW
    },
    Connection, Xid,
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

pub fn grab(conn: &Connection) -> Result<()> {
    Ok(())
}

pub fn get_active_internal(conn: &Connection) -> Result<ActiveWindowData> {
    let setup = conn.get_setup();
    let k = setup.roots().collect::<Vec<_>>();

    let focus_reply = conn.wait_for_reply(conn.send_request(&GetInputFocus {}))?;
    dbg!(&k.len());
    let mut wnd = focus_reply.focus();

    loop {
        let tree = conn.wait_for_reply(conn.send_request(&QueryTree {
            window: wnd,
        }))?;
        if wnd == tree.root() || tree.parent() == tree.root() {
            dbg!(wnd);
            break;
        }
        else {
            wnd = tree.parent();
        }
        



    }

    dbg!(&wnd);
    let hehe = conn.send_request(&QueryInfo {
        drawable: xcb::x::Drawable::Window(wnd),
    });
    let reply: QueryInfoReply = conn.wait_for_reply(hehe)?;
    dbg!(reply.ms_since_user_input());
    Ok(ActiveWindowData {
        title: "".into(),
        process_name: "".into(),
    })
}

pub fn get_active() -> Result<ActiveWindowData> {
    let (conn, screen_num) = xcb::Connection::connect(None)?;
    let _ = conn.send_request(&GrabServer {});

    let result = get_active_internal(&conn);
    let _ = conn.send_request(&UngrabServer {});
    result
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
