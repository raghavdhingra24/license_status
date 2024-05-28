mod chkr;

use std::collections::HashMap;
use std::os::raw::c_void;
use std::rc::Rc;
use std::thread;
use windows::Win32::Foundation::HWND;
use windows::Win32::UI::Controls::MARGINS;
use windows::core::{PCWSTR, HSTRING};
use windows::Win32::System::Registry::*;
use windows::Win32::Graphics::Dwm::*;
use i_slint_backend_winit::WinitWindowAccessor;
use slint::*;
use raw_window_handle::HasWindowHandle;
slint::include_modules!();

trait GetPcwstr {
    fn to_pcwstr(&self) -> PCWSTR;
}
impl GetPcwstr for str {
    fn to_pcwstr(&self) -> PCWSTR {
        let h_string = HSTRING::from(self);
        return PCWSTR(h_string.as_ptr())
    }
}

slint::slint!(import { ItemQueue, InfoStruct, InfoVal } from "./src/slint/main.slint";);

fn callback_handler(info: HashMap<String, String>,
                        window: Weak<MainWindow>) {
    slint::invoke_from_event_loop(move ||{
        let vals = Rc::new(VecModel::<InfoVal>::from(vec![]));
        let mut non_blank_vals = 1;
        let name = info.get("Name").unwrap().clone();
        for items in info {
            if items.0 != "Name" {
                let val = InfoVal {
                    sno: non_blank_vals,
                    name: items.0.into(),
                    value: items.1.into()
                };
                vals.push(val);
                non_blank_vals += 1;
            }
        }
        let window = window.upgrade().unwrap();
        let infos = InfoStruct {
            name: name.into(),
            props: vals.into()
        };
        let model_rc = window.get_card_info_list();
        let model = model_rc.as_any().downcast_ref::<VecModel<InfoStruct>>().unwrap();
        model.push(infos.into());
        window.set_btn_enabled(true)
    }).unwrap();
}

fn run_btn_pressed(win: Weak<MainWindow>) {
    let window = win.upgrade().unwrap();
    window.set_btn_enabled(false);
    let model_rc = window.get_card_info_list();
    let model = model_rc.as_any().downcast_ref::<VecModel<InfoStruct>>().unwrap();
    //let mut offset = 0;
    for _i in 0..(model.row_count()) {
        model.remove(0);
    }
    let win = window.as_weak();
    thread::spawn(|| {
        chkr::get_license_info(callback_handler, win)
    });

}

fn get_windows_build() -> i32 {
    unsafe {
        let mut value = vec![0; 16];
        let mut size = (std::mem::size_of::<u16>() * value.len()) as u32;
        let _ = RegGetValueW(HKEY_LOCAL_MACHINE,
            "SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion".to_pcwstr(),
            "CurrentBuildNumber".to_pcwstr(),
            RRF_RT_ANY,
            None,
            Some(value.as_mut_ptr() as *mut c_void),
            Some(&mut size)
        );
        let win_build:i32 = PCWSTR(&mut value[0]).to_string().unwrap().parse().unwrap();
        return win_build
    }
}

fn main() {
    let window = MainWindow::new().unwrap();
    let data = Rc::new(VecModel::<InfoStruct>::from(vec![]));
    window.set_card_info_list(data.clone().into());
    window.window().with_winit_window(|winit_window| {
        let win_id = winit_window.window_handle().unwrap().into();
        let win_build = get_windows_build();
        if win_build > 22523 {
            if let raw_window_handle::RawWindowHandle::Win32(win_id) = win_id {
                let hwnd = HWND(win_id.hwnd.get());
                unsafe {
                    let mut margins = MARGINS::default();
                    margins.cxLeftWidth = -1;
                    let _ = DwmExtendFrameIntoClientArea(hwnd, &margins);
                    let val = DWM_SYSTEMBACKDROP_TYPE(2);
                    let val_size = std::mem::size_of::<DWM_SYSTEMBACKDROP_TYPE>() as u32;
                    println!("{val_size}");
                    let r = DwmSetWindowAttribute(hwnd,
                        DWMWA_SYSTEMBACKDROP_TYPE,
                        std::ptr::addr_of!(val) as *const c_void,
                        val_size
                    );
                    let result = match r {
                        Ok(_res) => "Passed",
                        Err(err) => {println!("{:?}", err); "Error"},
                    };
                    if result == "Passed" {
                        window.set_bg_color(slint::Brush::SolidColor(slint::Color::from_argb_u8(0, 0, 0, 0)));
                    }
                }
            };
        } 
        // Uncomment for testing any change (in windows 10) for win32api
        /*
        else {
            if let raw_window_handle::RawWindowHandle::Win32(win_id) = win_id {
                let hwnd = HWND(win_id.hwnd.get());
                unsafe {
                    let val = BOOL(0);
                    let val_size = std::mem::size_of::<BOOL>() as u32;
                    println!("{val_size}");
                    let r = DwmSetWindowAttribute(hwnd,
                        DWMWA_USE_IMMERSIVE_DARK_MODE,
                        std::ptr::addr_of!(val) as *const c_void,
                        val_size
                    );
                    match r {
                        Ok(_res) => "Passed",
                        Err(err) => {println!("{:?}", err); "Error"},
                    };
                }
            }
        }
        */
    });
    let win: Weak<MainWindow> = window.as_weak();
    window.on_run_btn_clicked(move || {
        run_btn_pressed(win.clone());
    });
    window.run().unwrap();
}