#![cfg_attr(feature="clippy", feature(plugin))]
#![cfg_attr(feature="clippy", plugin(clippy))]

use std::fs::File;
use std::io::{Read, Write};
use std::path::{PathBuf, Path};
use std::thread;
use std::time::Duration;

const BACKLIGHT: &str = "gmux_backlight";
const KBD_BACKLIGHT: &str = "smc::kbd_backlight";
const APPLE_SMC: &str = "applesmc.768";
const POWER_SUPPLY: &str = "ADP1";

fn main() {
    let backlight_dir = Path::new("/sys/class/backlight").join(BACKLIGHT);
    let kbd_backlight_dir = Path::new("/sys/class/leds").join(KBD_BACKLIGHT);
    let psu_dir = Path::new("/sys/class/power_supply").join(POWER_SUPPLY);

    let backlight_file = backlight_dir.join("brightness");
    let kbd_backlight_file = kbd_backlight_dir.join("brightness");
    let light_file = Path::new("/sys/devices/platform").join(APPLE_SMC).join("light");

    let sys_max_brightness: f64 = read_file(&backlight_dir.join("max_brightness")).trim().parse().unwrap();

    let sys_max_kbd_brightness: f64 = read_file(&kbd_backlight_dir.join("max_brightness")).trim().parse().unwrap();
    let min_kbd_brightness = 0.0;

    let mut i = 0;

    loop {
        let light_value: f64 = read_file(&light_file)
            .trim()
            .trim_matches(|c| c == '(' || c == ')')
            .split(',')
            .next().unwrap()
            .parse().unwrap();

        let power_supply_online = read_file(&psu_dir.join("online")).trim().parse::<i32>().unwrap() == 1;
        let min_brightness = sys_max_brightness * (if power_supply_online { 0.1 } else { 0.025 });
        let max_brightness = sys_max_brightness * (if power_supply_online { 1.0 } else { 0.25 });
        let max_kbd_brightness = sys_max_kbd_brightness * (if power_supply_online { 0.2 } else { 0.05 });

        let mut new_brightness = (light_value * 32.0) / 255.0 * sys_max_brightness;
        if new_brightness < min_brightness {
            new_brightness = min_brightness;
        }
        else if new_brightness > max_brightness {
            new_brightness = max_brightness;
        }
        let mut backlight_fp = File::create(&backlight_file).unwrap();
        if let Err(e) = backlight_fp.write_all(&format!("{}\n", new_brightness as i32).into_bytes()) {
            println!("Failed to set display backlight brightness: {}", e);
        }

        let mut new_kbd_brightness = if light_value >= 5.0 { 0.0 } else { sys_max_kbd_brightness / 4.0 * (4.0 - light_value) };
        if new_kbd_brightness < min_kbd_brightness {
            new_kbd_brightness = min_kbd_brightness;
        }
        else if new_kbd_brightness > max_kbd_brightness {
            new_kbd_brightness = max_kbd_brightness;
        }
        let mut kbd_backlight_fp = File::create(&kbd_backlight_file).unwrap();
        if let Err(e) = kbd_backlight_fp.write_all(&format!("{}\n", new_kbd_brightness as i32).into_bytes()) {
            println!("Failed to set keyboard backlight brightness: {}", e);
        }

        if (i % 10) == 0 {
            println!("ambient: {}", light_value);
            println!("display: min={}, max={}, cur={}", min_brightness, max_brightness, new_brightness);
            println!("keyboard: min={}, max={}, cur={}", min_kbd_brightness, max_kbd_brightness, new_kbd_brightness);
            i = 0;
        }

        i += 1;
        thread::sleep(Duration::from_secs(10));
    }
}

fn read_file(filename: &PathBuf) -> String {
    let mut file = File::open(filename).unwrap();
    let mut content = String::new();
    file.read_to_string(&mut content).unwrap();
    content
}
