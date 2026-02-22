use hidapi::HidApi;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

const K380_SEQ_FKEYS_ON: &[u8] = &[0x10, 0xff, 0x0b, 0x1e, 0x00, 0x00, 0x00];
const K380_SEQ_FKEYS_OFF: &[u8] = &[0x10, 0xff, 0x0b, 0x1e, 0x01, 0x00, 0x00];

fn get_state_file_path() -> PathBuf {
    let mut path = env::temp_dir();
    path.push("k380_fn_state.txt");
    path
}

fn pause() {
    print!("\n请按 Enter 键退出...");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    let _ = io::stdin().read_line(&mut input);
}

fn main() {
    println!("======================================");
    println!("      罗技 K380 FN 键切换工具");
    println!("======================================");

    let state_file = get_state_file_path();
    let current_state = fs::read_to_string(&state_file).unwrap_or_else(|_| String::from("off"));
    let current_state = current_state.trim();

    let (target_state, sequence, target_desc) = if current_state == "on" {
        ("off", K380_SEQ_FKEYS_OFF, "多媒体键模式 (需加按 Fn 触发 F1-F12)")
    } else {
        ("on", K380_SEQ_FKEYS_ON, "F1-F12 标准模式 (适合办公/编程)")
    };

    println!("当前状态: {}", if current_state == "on" { "F1-F12 标准模式 [ON]" } else { "多媒体键模式 [OFF]" });
    println!("正在切换为 -> {}\n", target_desc);

    let api = match HidApi::new() {
        Ok(api) => api,
        Err(e) => {
            eprintln!("❌ 初始化系统接口失败: {}", e);
            pause();
            return;
        }
    };

    let mut device_found = false;
    let mut success = false;

    for device_info in api.device_list() {
        let is_k380 = (device_info.vendor_id() == 0x046d && device_info.product_id() == 0xb342) 
                   || device_info.product_string().unwrap_or("").contains("K380");

        // 🚨 核心修复点：严格过滤 Usage Page
        // 标准键盘是 0x01，多媒体是 0x0C。罗技的 HID++ 协议通道在厂商自定义页（0xFF00 或以上）。
        // 在某些 Windows 蓝牙驱动下可能读出为 0，所以排除掉已知的标准页。
        let usage_page = device_info.usage_page();
        let is_vendor_interface = usage_page >= 0xFF00 || usage_page == 0;

        if is_k380 && is_vendor_interface {
            device_found = true;
            
            if let Ok(device) = device_info.open_device(&api) {
                match device.write(sequence) {
                    Ok(bytes_written) if bytes_written > 0 => {
                        success = true;
                    }
                    _ => {} 
                }
            }
        }
    }

    if !device_found {
        eprintln!("❌ 未找到支持的罗技 K380 协议接口！请确认键盘已连接。");
    } else if success {
        if let Err(e) = fs::write(&state_file, target_state) {
            eprintln!("⚠️ 切换成功，但保存状态失败: {}", e);
        } else {
            println!("🎉 切换成功！");
            println!("现在的状态是: {}", if target_state == "on" { "[ON] F1-F12 优先" } else { "[OFF] 多媒体键优先" });
        }
    } else {
        eprintln!("⚠️ 找到了键盘，但由于权限被拒无法写入指令。");
    }

    pause();
}