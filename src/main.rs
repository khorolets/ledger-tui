use std::str::FromStr;

use cli_clipboard::{ClipboardContext, ClipboardProvider};
use cursive::traits::*;
use cursive::views::{Button, Dialog, DummyView, EditView, LinearLayout, SelectView};
use cursive::Cursive;

use near_ledger::get_public_key;
use slip10::BIP32Path;

fn main() {
    let mut siv = cursive::default();

    let select = SelectView::<String>::new()
        .on_submit(on_submit)
        .with_name("select")
        .fixed_size((60, 10));
    let buttons = LinearLayout::vertical()
        .child(Button::new("Fetch from HD path", add_name))
        .child(DummyView)
        .child(Button::new("Quit", Cursive::quit));

    siv.add_layer(
        Dialog::around(
            LinearLayout::horizontal()
                .child(select)
                .child(DummyView)
                .child(buttons),
        )
        .title("Select a PublicKey"),
    );

    siv.run();
}

fn add_name(s: &mut Cursive) {
    fn ok(s: &mut Cursive, hdpath_str: &str) {
        let hd_path = match BIP32Path::from_str(hdpath_str) {
            Err(err) => {
                s.add_layer(
                    Dialog::text(format!("{}", err))
                        .title("Failed tp convert HD Path to BIP32Path")
                        .button("Quit", |s| {
                            s.pop_layer();
                        }),
                );
                return;
            }
            Ok(hd_path) => hd_path,
        };
        let public_key = match get_public_key(hd_path) {
            Err(err) => {
                s.add_layer(
                    Dialog::text(format!("{:?}", err))
                        .title("Failed to get public key from ledger")
                        .button("OK", |s| {
                            s.pop_layer();
                        }),
                );
                return;
            }
            Ok(public_key) => public_key,
        };

        let public_key = near_crypto::PublicKey::ED25519(near_crypto::ED25519PublicKey::from(
            public_key.to_bytes(),
        ));

        s.call_on_name("select", |view: &mut SelectView<String>| {
            view.add_item_str(public_key.to_string())
        });
        s.pop_layer();
    }

    s.add_layer(
        Dialog::around(
            EditView::new()
                .on_submit(ok)
                .content("44'/397'/0'/0'/1'")
                .with_name("hdpath")
                .fixed_width(20),
        )
        .title("Enter HD Path, then confirm it on the Ledger")
        .button("Ok", |s| {
            let hdpath = s
                .call_on_name("hdpath", |view: &mut EditView| view.get_content())
                .unwrap();
            ok(s, &hdpath);
        })
        .button("Cancel", |s| {
            s.pop_layer();
        }),
    );
}

fn on_submit(s: &mut Cursive, public_key: &str) {
    let mut ctx = ClipboardContext::new().unwrap();
    ctx.set_contents(public_key.to_owned()).unwrap();
    s.add_layer(
        Dialog::text(format!(
            "PublicKey: {} is copied to your clipboard",
            public_key
        ))
        .title(format!("{}", public_key))
        .button("Got it!", |s| {
            s.pop_layer();
        }),
    );
}
