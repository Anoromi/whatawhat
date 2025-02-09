use std::ptr::null;

use anyhow::Result;
use xcb::{
    screensaver::{QueryInfo, QueryInfoReply},
    x::{
        self, Atom, GetInputFocus, GetProperty, GrabServer, InputFocus, InternAtom,
        InternAtomCookie, QueryTree, UngrabServer, Window, ATOM_WINDOW,
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

// pub fn get_active() -> Result<ActiveWindowData> {
//     let (conn, screen_num) = xcb::Connection::connect(None)?;
//     let setup = conn.get_setup();
//     let k = setup.roots().collect::<Vec<_>>();
//
//     let focus_reply = conn.wait_for_reply(conn.send_request(&GetInputFocus {}))?;
//     dbg!(&k.len());
//     let root = focus_reply.focus();
//     let active_window_cookie = conn.send_request(&InternAtom {
//         only_if_exists: false,
//         name: b"_NET_ACTIVE_WINDOW",
//     });
//     let active_window_reply = conn.wait_for_reply(active_window_cookie)?.atom();
//     let property_cookie = conn.send_request(&GetProperty {
//         delete: false,
//         window: root,
//         property: active_window_reply,
//         r#type: ATOM_WINDOW,
//         long_offset: 0,
//         long_length: 1,
//     });
//     let property = conn.wait_for_reply(property_cookie)?;
//     dbg!(property);
//     // println!("{:?}", k.root());
//
//     let hehe = conn.send_request(&QueryInfo {
//         drawable: xcb::x::Drawable::Window(root),
//     });
//     let reply: QueryInfoReply = conn.wait_for_reply(hehe)?;
//     dbg!(reply.ms_since_user_input());
//
//     Ok(ActiveWindowData {
//         title: "".into(),
//         process_name: "".into(),
//     })
// }

pub fn get_pid_atom(conn: &Connection) -> Result<Atom> {
    let hehe = conn.wait_for_reply(conn.send_request(&InternAtom {
        only_if_exists: false,
        name: b"_NET_WM_PID",
    }))?;
    Ok(hehe.atom())
}

pub fn get_active_internal(conn: &Connection) -> Result<ActiveWindowData> {
    let setup = conn.get_setup();
    let k = setup.roots().collect::<Vec<_>>();

    let focus_reply = conn.wait_for_reply(conn.send_request(&GetInputFocus {}))?;
    dbg!(&k.len());
    let mut wnd = focus_reply.focus();

    loop {
        let tree = conn.wait_for_reply(conn.send_request(&QueryTree { window: wnd }))?;
        if wnd == tree.root() || tree.parent() == tree.root() {
            dbg!(wnd);
            break;
        } else {
            wnd = tree.parent();
        }
    }

    dbg!(&wnd);

    let wm_name = conn.wait_for_reply(conn.send_request(&x::GetProperty {
        delete: false,
        window: wnd,
        property: x::ATOM_WM_NAME,
        r#type: x::ATOM_STRING,
        long_offset: 0,
        long_length: 0,
    }))?;
    let title = String::from_utf8(wm_name.value().to_vec()).expect("The WM_NAME property is not valid UTF-8");

    dbg!(&title);

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
