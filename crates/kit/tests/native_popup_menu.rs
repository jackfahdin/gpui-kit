//! Run explicitly with `cargo test -p gpui-kit --features test-support --test native_popup_menu`.
//! TestWindow has no separate restore bounds, so this regression needs a native window.

#[cfg(all(target_os = "windows", not(feature = "gpui-fast")))]
mod windows {
    use ::windows::Win32::Foundation::HWND;
    use ::windows::Win32::UI::WindowsAndMessaging::{SW_RESTORE, ShowWindowAsync};
    use gpui_kit::component::Theme;
    use gpui_kit::component::menu::{PopupMenu, PopupMenuAppearance, PopupMenuItem};
    use gpui_kit::test::TestWindowExt as _;
    use gpui_kit::{
        App, AppContext as _, Context, Entity, IntoElement, Pixels, QuitMode, Render, Window,
        WindowBounds, WindowOptions, px, size,
    };
    use raw_window_handle::{HasWindowHandle as _, RawWindowHandle};
    use std::process::{Command, Stdio};
    use std::thread;
    use std::time::{Duration, Instant};

    struct MenuProbe {
        popup: Entity<PopupMenu>,
    }

    impl Render for MenuProbe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.popup.clone()
        }
    }

    fn menu(window: &mut Window, cx: &mut App, max_height: Option<Pixels>) -> Entity<PopupMenu> {
        PopupMenu::build(window, cx, |menu, _, _| {
            let menu = (0..40).fold(
                menu.appearance(PopupMenuAppearance::new().row_height(px(28.)))
                    .scrollable(true),
                |menu, index| menu.item(PopupMenuItem::new(format!("Theme {index}"))),
            );
            match max_height {
                Some(height) => menu.max_h(height),
                None => menu,
            }
        })
    }

    fn run_application() {
        gpui_kit::application()
            .with_quit_mode(QuitMode::Explicit)
            .run(|cx| {
                gpui_kit::init(cx);
                let (handle, popup) = gpui_kit::open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::centered(None, size(px(640.), px(320.)), cx)),
                        focus: false,
                        ..Default::default()
                    },
                    cx,
                    |window, cx| cx.new(|cx| MenuProbe { popup: menu(window, cx, None) }),
                )
                .expect("open native popup window");

                cx.spawn(async move |cx| {
                    for (phase, maximized) in [false, true, false].into_iter().enumerate() {
                        let started = Instant::now();
                        loop {
                            cx.background_executor()
                                .timer(Duration::from_millis(50))
                                .await;
                            let ready = handle
                                .update(cx, |_, window, _| {
                                    window.is_maximized() == maximized
                                        && window.viewport_size() == window.bounds().size
                                })
                                .expect("wait for native resize");
                            if ready {
                                break;
                            }
                            assert!(started.elapsed() < Duration::from_secs(5), "resize timed out");
                        }

                        handle
                            .update(cx, |_, window, cx| {
                                let height = window.bounds().size.height;
                                let restore_height = window.window_bounds().get_bounds().size.height;
                                if maximized {
                                    assert!(height > restore_height + px(100.));
                                } else {
                                    assert!(height < px(400.));
                                }
                                for rem in [12., 16., 24.] {
                                    Theme::global_mut(cx).font_size = px(rem);
                                    Theme::sync_base(cx);
                                    // Reopen the menu after each state/zoom change.
                                    popup.update(cx, |probe, cx| probe.popup = menu(window, cx, None));
                                    window.render_frame(cx);
                                    let actual = window.find("popup-menu").bounds().size.height;
                                    let expected = (height * 0.5).min(px(28.125 * rem));
                                    assert!(
                                        (actual - expected).abs() <= px(1.),
                                        "maximized={maximized}, rem={rem}, current={height:?}, \
                                         restore={restore_height:?}, menu={actual:?}, expected={expected:?}"
                                    );
                                    println!("PASS maximized={maximized} rem={rem} menu={actual:?}");
                                }

                                popup.update(cx, |probe, cx| probe.popup = menu(window, cx, Some(px(176.))));
                                window.render_frame(cx);
                                assert_eq!(window.find("popup-menu").bounds().size.height, px(176.));
                                // Restore the default before the next native transition.
                                popup.update(cx, |probe, cx| probe.popup = menu(window, cx, None));
                                if phase == 0 {
                                    window.zoom_window();
                                } else if phase == 1 {
                                    // This GPUI revision's zoom API only maximizes.
                                    let RawWindowHandle::Win32(handle) = window
                                        .window_handle()
                                        .expect("native window handle")
                                        .as_raw()
                                    else {
                                        panic!("expected a Win32 window");
                                    };
                                    unsafe {
                                        ShowWindowAsync(HWND(handle.hwnd.get() as *mut _), SW_RESTORE)
                                            .expect("restore native window");
                                    }
                                }
                            })
                            .expect("check native popup layout");
                    }
                    handle
                        .update(cx, |_, window, _| window.remove_window())
                        .expect("close native popup window");
                    cx.update(|cx| cx.quit());
                })
                .detach();
            });
        println!("native popup checks completed");
    }

    pub fn run() {
        if std::env::args().any(|arg| arg == "--native-popup-child") {
            run_application();
            return;
        }
        // Bound failures in native callbacks and teardown independently of Cargo.
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .arg("--native-popup-child")
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start native popup child");
        let started = Instant::now();
        loop {
            if child.try_wait().expect("poll native popup child").is_some() {
                break;
            }
            if started.elapsed() > Duration::from_secs(25) {
                child.kill().expect("stop hung native popup child");
                break;
            }
            thread::sleep(Duration::from_millis(20));
        }
        let output = child.wait_with_output().expect("reap native popup child");
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(
            output.status.success() && stdout.contains("native popup checks completed"),
            "status={}\n{stdout}\n{}",
            output.status,
            String::from_utf8_lossy(&output.stderr)
        );
        print!("{stdout}");
    }
}

fn main() {
    #[cfg(all(target_os = "windows", not(feature = "gpui-fast")))]
    windows::run();
    #[cfg(not(all(target_os = "windows", not(feature = "gpui-fast"))))]
    println!("native popup regression requires Windows with the native GPUI backend");
}
