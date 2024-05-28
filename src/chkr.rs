use std::{collections::HashMap, process::*};

pub fn get_license_info(callback: fn(HashMap<String, String>, crate::Weak<crate::MainWindow>) -> (),
                            window: crate::Weak<crate::MainWindow>) {
    let args = ["c:\\Windows\\System32\\slmgr.vbs","/dlv", "all"];
    let cmd = Command::new("cscript").args(args).output().expect("Not Found");
    
    let output = String::from_utf8_lossy(&cmd.stdout);
    let split = output.split("\r\n\r");
    for item in split {
        let mut val_map:HashMap<String, String> = HashMap::new();
        for items in item.split('\n'){
            if items.contains(": ") {
                let sv: Vec<_> = items.split(": ").collect();
                let body = sv[1].trim().to_string();
                if body.len() > 1 {
                    val_map.insert(String::from(sv[0]), body);
                }
            }
        }
        if val_map.contains_key("Name") {
            callback(val_map.clone(), window.clone());
        }
    }
}
