use cfg_if::cfg_if;

#[cfg(target_os = "windows")]
pub mod winstuff;

fn main() {
    let k = {
        cfg_if!(
            if #[cfg(target_os = "windows")] {
                {
                    use winstuff::hello_there;
                    hello_there()
                }
            }
            else {
                let g = 3;
                g.saturating_add(34);
                3
            }
        )
    };
}
