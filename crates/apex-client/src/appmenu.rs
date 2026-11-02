//! Off the Mac, the menu bar's menus. gpui has no menu bar there to put
//! `shell::menus` in, and without one Themes, Fonts, Restart Server and
//! the rest could be had only by their keys, if they have any: so a
//! button at the title bar's left, where the Mac's window buttons are,
//! drops the same menus down, each opening to the right under the
//! pointer as a menu bar's submenus do. A choice is dispatched as the
//! menu bar's would be, with the mark and the keys the Mac's shows.

use gpui::{deferred, div, prelude::*, px, rgb, AnyElement, Context, MenuItem, MouseButton, Window};

use crate::app::Acme;

/// A row's height, a rule's, the panel's padding, and its width: every
/// panel one width, so a submenu's place is known before it is laid out.
const ROW_H: f32 = 26.;
const RULE_H: f32 = 9.;
const PAD: f32 = 5.;
const PANEL_W: f32 = 260.;

impl Acme {
    /// The menu's button, three bars, and the menus under it while open:
    /// nothing on the Mac, whose menu bar has them.
    pub fn app_menu_button(&self, window: &mut Window, cx: &mut Context<Self>) -> Option<AnyElement> {
        if cfg!(target_os = "macos") {
            return None;
        }
        let t = crate::theme::theme();
        let hover = crate::theme::step(crate::text_element::ground(&t), 1);
        let open = self.app_menu.is_some();
        let button = div()
            .id("app-menu")
            .size(px(24.))
            .rounded(px(5.))
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(3.))
            .cursor_default()
            .when(open, move |d| d.bg(rgb(hover)))
            .hover(move |s| s.bg(rgb(hover)))
            .children((0..3).map(|_| div().w(px(14.)).h(px(1.5)).rounded(px(0.75)).bg(rgb(t.text_dim))))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    this.app_menu = if this.app_menu.is_some() { None } else { Some(Vec::new()) };
                    this.session_menu = false;
                    cx.notify();
                    cx.stop_propagation();
                }),
            );
        // the menu bar's titles, Apex File Edit View, as the first panel's
        // rows, each a submenu
        let panel = self.app_menu.clone().map(|open| {
            let top = crate::shell::menus().into_iter().map(MenuItem::Submenu).collect();
            deferred(self.app_menu_panel(top, 0, &open, window, cx).top(px(28.)).left(px(0.))).with_priority(2)
        });
        Some(div().size_full().flex().items_center().justify_center().child(div().relative().child(button).children(panel)).into_any_element())
    }

    /// A menu's panel, `depth` down the cascade, and the submenu `open`
    /// says is open from it beside it, level with its row.
    fn app_menu_panel(&self, items: Vec<MenuItem>, depth: usize, open: &[usize], window: &mut Window, cx: &mut Context<Self>) -> gpui::Stateful<gpui::Div> {
        let t = crate::theme::theme();
        let shadow = gpui::BoxShadow { color: gpui::hsla(0., 0., 0., 0.18), offset: gpui::point(px(0.), px(6.)), blur_radius: px(18.), spread_radius: px(0.), inset: false };
        let mut panel = div()
            .id(("app-menu-panel", depth))
            .absolute()
            .w(px(PANEL_W))
            .p(px(PAD))
            .rounded(px(9.))
            .bg(rgb(t.panel_bg))
            .border_1()
            .border_color(rgb(t.panel_border))
            .shadow(vec![shadow])
            .flex()
            .flex_col()
            .text_size(px(13.))
            .font_family(crate::fonts::ui())
            .child(self.overlay_mark())
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation());
        // a choice runs a command: B2's wash and ink, as the B4 menu's
        let (wash, ink) = t.sweep(true);
        let row = |id: usize, lit: bool| {
            div()
                .id(("app-menu-row", depth * 1000 + id))
                .flex_none()
                .h(px(ROW_H))
                .px(px(8.))
                .rounded(px(6.))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(8.))
                .cursor_default()
                .text_color(rgb(if lit { ink } else { t.panel_text }))
                .when(lit, |d| d.bg(rgb(wash)))
                .hover(move |s| s.bg(rgb(wash)).text_color(rgb(ink)))
        };
        let above: Vec<usize> = open[..depth.min(open.len())].to_vec();
        let chosen = open.get(depth).copied();
        let mut y = PAD;
        let mut sub = None;
        for (i, item) in items.into_iter().enumerate() {
            match item {
                MenuItem::Separator => {
                    panel = panel.child(div().flex_none().h(px(1.)).mx(px(6.)).my(px((RULE_H - 1.) / 2.)).bg(rgb(t.panel_border)));
                    y += RULE_H;
                }
                MenuItem::Submenu(m) => {
                    let here = chosen == Some(i);
                    let to: Vec<usize> = above.iter().copied().chain([i]).collect();
                    panel = panel.child(
                        row(i, here)
                            .child(div().flex_none().w(px(12.)))
                            .child(div().flex_1().min_w_0().truncate().child(m.name.clone()))
                            .child(div().flex_none().child("›"))
                            .on_hover(cx.listener(move |this, on: &bool, _, cx| {
                                if *on && this.app_menu.as_ref() != Some(&to) {
                                    this.app_menu = Some(to.clone());
                                    cx.notify();
                                }
                            })),
                    );
                    if here {
                        sub = Some((m.items, y));
                    }
                    y += ROW_H;
                }
                MenuItem::Action { name, action, checked, .. } => {
                    // the Mac's menus mark the choice in the name
                    let (name, mark) = match name.strip_suffix(" ✓") {
                        Some(n) => (n.to_string(), true),
                        None => (name.to_string(), checked),
                    };
                    let keys = window.bindings_for_action(action.as_ref()).first().map(say);
                    let to = above.clone();
                    panel = panel.child(
                        row(i, false)
                            .child(div().flex_none().w(px(12.)).child(if mark { "✓" } else { "" }))
                            .child(div().flex_1().min_w_0().truncate().child(name))
                            .when_some(keys, |d, k| d.child(div().flex_none().text_size(px(12.)).opacity(0.6).child(k)))
                            .on_hover(cx.listener(move |this, on: &bool, _, cx| {
                                if *on && this.app_menu.as_ref() != Some(&to) {
                                    this.app_menu = Some(to.clone());
                                    cx.notify();
                                }
                            }))
                            .on_mouse_down(
                                MouseButton::Left,
                                cx.listener(move |this, _, window, cx| {
                                    this.app_menu = None;
                                    window.dispatch_action(action.boxed_clone(), cx);
                                    cx.notify();
                                    cx.stop_propagation();
                                }),
                            ),
                    );
                    y += ROW_H;
                }
                MenuItem::SystemMenu(_) => {}
            }
        }
        if let Some((items, y)) = sub {
            panel = panel.child(self.app_menu_panel(items, depth + 1, open, window, cx).top(px(y - PAD)).left(px(PANEL_W - 6.)));
        }
        panel
    }
}

/// A binding as a Linux menu writes it: `Ctrl+Shift+N`.
fn say(b: &gpui::KeyBinding) -> String {
    let one = |k: &gpui::KeybindingKeystroke| {
        let m = k.modifiers();
        let mut s = String::new();
        for (on, word) in [(m.control, "Ctrl+"), (m.alt, "Alt+"), (m.shift, "Shift+"), (m.platform, "Super+")] {
            if on {
                s.push_str(word);
            }
        }
        let mut key = k.key().chars();
        if let Some(c) = key.next() {
            s.extend(c.to_uppercase());
            s.push_str(key.as_str());
        }
        s
    };
    b.keystrokes().iter().map(one).collect::<Vec<_>>().join(" ")
}
